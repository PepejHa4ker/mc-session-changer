use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JObjectArray};
use serde::{Deserialize, Serialize};
use crate::jvm::get_jvm;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub modifiers: i32,
    pub modifiers_str: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MethodInfo {
    pub name: String,
    pub signature: String,
    pub modifiers: i32,
    pub modifiers_str: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub class_name: String,
    pub simple_name: String,
    pub superclass: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub modifiers: i32,
    pub modifiers_str: String,
}

pub struct JvmAnalyzer;

impl JvmAnalyzer {
    pub unsafe fn analyze_class(env: &mut JNIEnv, class_name: &str) -> anyhow::Result<ClassInfo> {
        env.push_local_frame(512)?;

        // Преобразуем имя класса в формат Java (заменяем . на /)
        let jvm_class_name = class_name.replace('.', "/");

        let jvm = get_jvm();
        let jclass = match jvm.forge_find_class(env, &jvm_class_name) {
            Some(cls) => cls,
            None => {
                let _ = env.pop_local_frame(&JObject::null());
                anyhow::bail!("Class not found: {}", class_name);
            }
        };

        // Получаем информацию о классе
        let simple_name = get_simple_class_name(env, &jclass)?;
        let superclass = get_superclass_name(env, &jclass)?;
        let interfaces = get_interfaces_names(env, &jclass)?;
        let modifiers = get_class_modifiers(env, &jclass)?;
        let modifiers_str = format_modifiers(modifiers, true);

        // Получаем поля
        let fields = get_class_fields(env, &jclass)?;

        // Получаем методы
        let methods = get_class_methods(env, &jclass)?;

        let _ = env.pop_local_frame(&JObject::null());

        Ok(ClassInfo {
            class_name: class_name.to_string(),
            simple_name,
            superclass,
            interfaces,
            fields,
            methods,
            modifiers,
            modifiers_str,
        })
    }
}

unsafe fn get_simple_class_name(env: &mut JNIEnv, jclass: &JClass) -> anyhow::Result<String> {
    let simple_name = env
        .call_method(jclass, "getSimpleName", "()Ljava/lang/String;", &[])?
        .l()?;

    if simple_name.is_null() {
        return Ok(String::new());
    }

    let result: String = env.get_string(&JString::from(simple_name))?.into();
    Ok(result)
}

unsafe fn get_superclass_name(
    env: &mut JNIEnv,
    jclass: &JClass,
) -> anyhow::Result<Option<String>> {
    let superclass = env
        .call_method(jclass, "getSuperclass", "()Ljava/lang/Class;", &[])?
        .l()?;

    if superclass.is_null() {
        return Ok(None);
    }

    let name = env
        .call_method(&superclass, "getName", "()Ljava/lang/String;", &[])?
        .l()?;

    if name.is_null() {
        return Ok(None);
    }

    let result: String = env.get_string(&JString::from(name))?.into();
    Ok(Some(result))
}

unsafe fn get_interfaces_names(env: &mut JNIEnv, jclass: &JClass) -> anyhow::Result<Vec<String>> {
    let interfaces_array = env
        .call_method(jclass, "getInterfaces", "()[Ljava/lang/Class;", &[])?
        .l()?;

    if interfaces_array.is_null() {
        return Ok(Vec::new());
    }

    let interfaces_obj_array: JObjectArray = interfaces_array.into();
    let length = env.get_array_length(&interfaces_obj_array)? as usize;
    let mut interfaces = Vec::new();

    for i in 0..length {
        let interface_class = env
            .get_object_array_element(&interfaces_obj_array, i as i32)?;

        let name = env
            .call_method(&interface_class, "getName", "()Ljava/lang/String;", &[])?
            .l()?;

        if !name.is_null() {
            let iface_name: String = env.get_string(&JString::from(name))?.into();
            interfaces.push(iface_name);
        }
    }

    Ok(interfaces)
}

unsafe fn get_class_modifiers(env: &mut JNIEnv, jclass: &JClass) -> anyhow::Result<i32> {
    let modifiers = env
        .call_method(jclass, "getModifiers", "()I", &[])?
        .i()?;

    Ok(modifiers)
}

unsafe fn get_class_fields(
    env: &mut JNIEnv,
    jclass: &JClass,
) -> anyhow::Result<Vec<FieldInfo>> {
    let fields_array = env
        .call_method(jclass, "getDeclaredFields", "()[Ljava/lang/reflect/Field;", &[])?
        .l()?;

    if fields_array.is_null() {
        return Ok(Vec::new());
    }

    let fields_obj_array: JObjectArray = fields_array.into();
    let length = env.get_array_length(&fields_obj_array)? as usize;
    let mut fields = Vec::new();

    for i in 0..length {
        let field_obj = env
            .get_object_array_element(&fields_obj_array, i as i32)?;

        // Получаем имя поля
        let name = env
            .call_method(&field_obj, "getName", "()Ljava/lang/String;", &[])?
            .l()?;
        let name_str: String = if !name.is_null() {
            env.get_string(&JString::from(name))?.into()
        } else {
            "unknown".to_string()
        };

        // Получаем тип поля
        let field_type = env
            .call_method(&field_obj, "getType", "()Ljava/lang/Class;", &[])?
            .l()?;
        let type_str = if !field_type.is_null() {
            let type_name = env
                .call_method(&field_type, "getName", "()Ljava/lang/String;", &[])?
                .l()?;
            if !type_name.is_null() {
                env.get_string(&JString::from(type_name))?.into()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        // Получаем модификаторы
        let modifiers = env
            .call_method(&field_obj, "getModifiers", "()I", &[])?
            .i()?;
        let modifiers_str = format_modifiers(modifiers, false);

        fields.push(FieldInfo {
            name: name_str,
            field_type: type_str,
            modifiers,
            modifiers_str,
        });
    }

    Ok(fields)
}

unsafe fn get_class_methods(
    env: &mut JNIEnv,
    jclass: &JClass,
) -> anyhow::Result<Vec<MethodInfo>> {
    let methods_array = env
        .call_method(jclass, "getDeclaredMethods", "()[Ljava/lang/reflect/Method;", &[])?
        .l()?;

    if methods_array.is_null() {
        return Ok(Vec::new());
    }

    let methods_obj_array: JObjectArray = methods_array.into();
    let length = env.get_array_length(&methods_obj_array)? as usize;
    let mut methods = Vec::new();

    for i in 0..length {
        let method_obj = env
            .get_object_array_element(&methods_obj_array, i as i32)?;

        // Получаем имя метода
        let name = env
            .call_method(&method_obj, "getName", "()Ljava/lang/String;", &[])?
            .l()?;
        let name_str: String = if !name.is_null() {
            env.get_string(&JString::from(name))?.into()
        } else {
            "unknown".to_string()
        };

        // Получаем сигнатуру метода
        let signature = env
            .call_method(&method_obj, "toString", "()Ljava/lang/String;", &[])?
            .l()?;
        let signature_str: String = if !signature.is_null() {
            env.get_string(&JString::from(signature))?.into()
        } else {
            "unknown".to_string()
        };

        // Получаем модификаторы
        let modifiers = env
            .call_method(&method_obj, "getModifiers", "()I", &[])?
            .i()?;
        let modifiers_str = format_modifiers(modifiers, false);

        methods.push(MethodInfo {
            name: name_str,
            signature: signature_str,
            modifiers,
            modifiers_str,
        });
    }

    Ok(methods)
}

fn format_modifiers(modifiers: i32, is_class: bool) -> String {
    let mut result = Vec::new();

    // Java модификаторы
    const PUBLIC: i32 = 0x0001;
    const FINAL: i32 = 0x0010;
    const STATIC: i32 = 0x0008;
    const PROTECTED: i32 = 0x0004;
    const PRIVATE: i32 = 0x0002;
    const ABSTRACT: i32 = 0x0400;
    const VOLATILE: i32 = 0x0040;
    const TRANSIENT: i32 = 0x0080;
    const SYNCHRONIZED: i32 = 0x0020;
    const NATIVE: i32 = 0x0100;
    const STRICTFP: i32 = 0x0800;
    const INTERFACE: i32 = 0x0200;

    if modifiers & PUBLIC != 0 {
        result.push("public");
    }
    if modifiers & PROTECTED != 0 {
        result.push("protected");
    }
    if modifiers & PRIVATE != 0 {
        result.push("private");
    }
    if modifiers & STATIC != 0 {
        result.push("static");
    }
    if modifiers & FINAL != 0 {
        result.push("final");
    }
    if modifiers & ABSTRACT != 0 {
        result.push("abstract");
    }
    if modifiers & SYNCHRONIZED != 0 {
        result.push("synchronized");
    }
    if modifiers & VOLATILE != 0 {
        result.push("volatile");
    }
    if modifiers & TRANSIENT != 0 {
        result.push("transient");
    }
    if modifiers & NATIVE != 0 {
        result.push("native");
    }
    if modifiers & STRICTFP != 0 {
        result.push("strictfp");
    }
    if is_class && modifiers & INTERFACE != 0 {
        result.push("interface");
    }

    result.join(" ")
}