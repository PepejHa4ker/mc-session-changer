use jni::objects::{JClass, JObject, JString, JValue};
use jni::{AttachGuard, JNIEnv, JavaVM};
use parking_lot::Mutex;
use std::ptr::null_mut;
use std::sync::OnceLock;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use crate::hooks::setup_jni_hooks;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub player_id: String,
    pub access_token: String,
    pub session_type: String,
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            username: "Player".to_string(),
            player_id: "00000000-0000-0000-0000-000000000000".to_string(),
            access_token: "0".to_string(),
            session_type: "mojang".to_string(),
        }
    }
}

pub struct JvmInfo {
    pub jvm: Option<JavaVM>,
    current_session: Mutex<SessionInfo>,
}

static MINECRAFT_SESSION: OnceLock<JvmInfo> = OnceLock::new();
static mut JVM_HOOKS_INITIALIZED: bool = false;

impl JvmInfo {
    pub fn new() -> Self {
        Self {
            jvm: None,
            current_session: Mutex::new(SessionInfo::default()),
        }
    }
    

    pub fn initialize(&mut self) -> Result<(), String> {
        unsafe {
            let jvm_dll = GetModuleHandleA(b"jvm.dll\0".as_ptr() as *const i8);
            if jvm_dll.is_null() {
                return Err("jvm.dll not found".to_string());
            }

            let getter_ptr =
                GetProcAddress(jvm_dll, b"JNI_GetCreatedJavaVMs\0".as_ptr() as *const i8);
            if getter_ptr.is_null() {
                return Err("JNI_GetCreatedJavaVMs not found".to_string());
            }

            type GetCreatedJavaVMs = unsafe extern "C" fn(
                vm_buf: *mut *mut jni::sys::JavaVM,
                buf_len: jni::sys::jsize,
                n_vms: *mut jni::sys::jsize,
            ) -> i32;

            let jvm_getter: GetCreatedJavaVMs = std::mem::transmute(getter_ptr);

            let mut count: jni::sys::jsize = 0;
            let result = jvm_getter(null_mut(), 0, &mut count);
            if result != 0 || count <= 0 {
                return Err("No JavaVM found".to_string());
            }

            let mut buffer = vec![null_mut(); count as usize];
            let result = jvm_getter(buffer.as_mut_ptr(), count, &mut count);
            if result != 0 {
                return Err("Failed to get JavaVM instances".to_string());
            }

            self.jvm = Some(
                JavaVM::from_raw(buffer[0])
                    .map_err(|e| format!("Invalid JavaVM pointer: {:?}", e))?,
            );

            if let Ok(session_info) = self.load_current_session() {
                *self.current_session.lock() = session_info;
            }

            Ok(())
        }
    }

    pub fn get_env(&'_ self) -> Result<AttachGuard<'_>, String> {
        if let Some(jvm) = &self.jvm {
            jvm.attach_current_thread()
                .map_err(|e| format!("Failed to attach thread: {:?}", e))
        } else {
            tracing::error!("JVM not initialized");
            Err("JVM not initialized".to_string())
        }
    }

    pub fn get_jvm(&self) -> Option<&JavaVM> {
        self.jvm.as_ref()
    }

    fn find_forge_launch_class_loader<'a>(&self, env: &mut JNIEnv<'a>) -> Option<JObject<'a>> {
        let launch_class = env.find_class("net/minecraft/launchwrapper/Launch").ok()?;
        let class_loader_field = env
            .get_static_field(
                &launch_class,
                "classLoader",
                "Lnet/minecraft/launchwrapper/LaunchClassLoader;",
            )
            .ok()?;

        let class_loader_obj = class_loader_field.l().ok()?;
        if class_loader_obj.is_null() {
            return None;
        }

        Some(class_loader_obj)
    }

    pub fn forge_find_class<'a>(&self, env: &mut JNIEnv<'a>, class_name: &str) -> Option<JClass<'a>> {
        let launch_class_loader = self.find_forge_launch_class_loader(env)?;
        let class_name_jstr = env.new_string(class_name).ok()?;

        let class_result = env
            .call_method(
                launch_class_loader,
                "findClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(&class_name_jstr)],
            )
            .ok()?;

        let class_obj = class_result.l().ok()?;
        if class_obj.is_null() {
            return None;
        }

        Some(JClass::from(class_obj))
    }

    fn get_minecraft_instance<'a>(&self, env: &mut JNIEnv<'a>) -> Option<JObject<'a>> {
        let minecraft_class = self.forge_find_class(env, "net.minecraft.client.Minecraft")?;
       
        unsafe {
            if !JVM_HOOKS_INITIALIZED {
                JVM_HOOKS_INITIALIZED = true;
                setup_jni_hooks(self.get_jvm().unwrap().get_java_vm_pointer()).expect("a");
            }
        }
        let minecraft_instance = env
            .call_static_method(
                minecraft_class,
                "func_71410_x",
                "()Lnet/minecraft/client/Minecraft;",
                &[],
            )
            .ok()?;

        let instance_obj = minecraft_instance.l().ok()?;

        if instance_obj.is_null() {
            return None;
        }

        Some(instance_obj)
    }

    fn load_current_session(&self) -> Result<SessionInfo, String> {
        let mut env = self.get_env()?;

        let minecraft_instance = self
            .get_minecraft_instance(&mut env)
            .ok_or("Failed to get Minecraft instance")?;

        let session_field = env
            .get_field(
                minecraft_instance,
                "field_71449_j",
                "Lnet/minecraft/util/Session;",
            )
            .map_err(|e| format!("Failed to get session field: {:?}", e))?;

        let session_obj = session_field
            .l()
            .map_err(|e| format!("Failed to get session object: {:?}", e))?;

        if session_obj.is_null() {
            return Err("Session object is null".to_string());
        }

        let mut session_info = SessionInfo::default();

        if let Ok(username_field) =
            env.get_field(&session_obj, "field_74286_b", "Ljava/lang/String;")
        {
            if let Ok(username_obj) = username_field.l() {
                if !username_obj.is_null() {
                    let username_str = JString::from(username_obj);
                    if let Ok(username_string) = env.get_string(&username_str) {
                        session_info.username = username_string.into();
                    }
                }
            }
        }

        if let Ok(player_id_field) =
            env.get_field(&session_obj, "field_148257_b", "Ljava/lang/String;")
        {
            if let Ok(player_id_obj) = player_id_field.l() {
                if !player_id_obj.is_null() {
                    let player_id_str = JString::from(player_id_obj);
                    if let Ok(player_id_string) = env.get_string(&player_id_str) {
                        session_info.player_id = player_id_string.into();
                    }
                }
            }
        }

        if let Ok(access_token_field) =
            env.get_field(&session_obj, "field_148258_c", "Ljava/lang/String;")
        {
            if let Ok(access_token_obj) = access_token_field.l() {
                if !access_token_obj.is_null() {
                    let access_token_str = JString::from(access_token_obj);
                    if let Ok(access_token_string) = env.get_string(&access_token_str) {
                        session_info.access_token = access_token_string.into();
                    }
                }
            }
        }

        if let Ok(session_type_field) = env.get_field(
            &session_obj,
            "field_152429_d",
            "Lnet/minecraft/util/Session$Type;",
        ) {
            if let Ok(session_type_obj) = session_type_field.l() {
                if !session_type_obj.is_null() {
                    if let Ok(type_string_field) =
                        env.get_field(session_type_obj, "field_152426_d", "Ljava/lang/String;")
                    {
                        if let Ok(type_string_obj) = type_string_field.l() {
                            if !type_string_obj.is_null() {
                                let type_string_str = JString::from(type_string_obj);
                                if let Ok(type_string) = env.get_string(&type_string_str) {
                                    session_info.session_type = type_string.into();
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(session_info)
    }

    fn update_minecraft_session(&self, new_session: &SessionInfo) -> Result<(), String> {
        let mut env = self.get_env()?;

        let minecraft_instance = self
            .get_minecraft_instance(&mut env)
            .ok_or("Failed to get Minecraft instance")?;

        let session_class = self
            .forge_find_class(&mut env, "net.minecraft.util.Session")
            .ok_or("Failed to load Session class")?;

        let username_jstr = env
            .new_string(&new_session.username)
            .map_err(|e| format!("Failed to create username string: {:?}", e))?;
        let player_id_jstr = env
            .new_string(&new_session.player_id)
            .map_err(|e| format!("Failed to create player ID string: {:?}", e))?;
        let access_token_jstr = env
            .new_string(&new_session.access_token)
            .map_err(|e| format!("Failed to create access token string: {:?}", e))?;

        let session_type_jstr = env
            .new_string(&new_session.session_type)
            .map_err(|e| format!("Failed to create session type string: {:?}", e))?;

        let new_session_obj = env
            .new_object(
                session_class,
                "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&username_jstr),
                    JValue::Object(&player_id_jstr),
                    JValue::Object(&access_token_jstr),
                    JValue::Object(&session_type_jstr),
                ],
            )
            .map_err(|e| format!("Failed to create new session object: {:?}", e))?;

        env.set_field(
            minecraft_instance,
            "field_71449_j",
            "Lnet/minecraft/util/Session;",
            JValue::Object(&new_session_obj),
        )
            .map_err(|e| format!("Failed to set session field: {:?}", e))?;

        Ok(())
    }

    pub fn get_current_session(&self) -> SessionInfo {
        self.current_session.lock().clone()
    }

    pub fn refresh_session(&self) -> Result<(), String> {
        let session_info = self.load_current_session()?;
        *self.current_session.lock() = session_info;
        Ok(())
    }

    pub fn change_session(&self, new_session: SessionInfo) -> Result<(), String> {
        self.update_minecraft_session(&new_session)?;
        *self.current_session.lock() = new_session;
        Ok(())
    }
}

pub fn get_jvm() -> &'static JvmInfo {
    MINECRAFT_SESSION.get_or_init(|| {
        let mut session = JvmInfo::new();
        let _ = session.initialize();
        session
    })
}