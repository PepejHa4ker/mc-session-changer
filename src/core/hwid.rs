use crate::core::state::GlobalState;
use crate::utils::generate_hwid;
use anyhow::{Context, Result};
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub static HWID_SPOOFER: HwidSpoofer = HwidSpoofer::new();

pub struct HwidSpoofer {
    debounce_ms: u64,
    last_notification_ms: AtomicU64,
}

impl HwidSpoofer {
    pub const fn new() -> Self {
        Self {
            debounce_ms: 5_000,
            last_notification_ms: AtomicU64::new(0),
        }
    }

    pub fn write_hwid(&self, env: &mut JNIEnv, data_output: &JObject) -> Result<()> {
        let hwid = generate_hwid();
        for h in hwid {
            let to_write = format!("\u{1}{}", h);
            let jstr = env
                .new_string(to_write)
                .context("Failed to allocate Java string for HWID segment")?;
            let jstr_obj = jstr.into();

            env.call_method(
                data_output,
                "writeUTF",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&jstr_obj)],
            )
            .context("Failed to call DataOutput#writeUTF for HWID")?;
        }
        Ok(())
    }

    pub fn notify_success(&self) {
        let now_ms = now_millis();
        if !self.should_emit_notification(now_ms) {
            tracing::debug!("Skipping HWID notification due to debounce window");
            return;
        }

        self.last_notification_ms.store(now_ms, Ordering::Relaxed);

        if let Some(context_mutex) = GlobalState::instance().get_context().get() {
            if let Some(mut context_guard) = context_mutex.try_lock() {
                if let Some(context) = context_guard.as_mut() {
                    context
                        .notification_manager
                        .show_success("HWID Spoofed", "Spoofed hardware identifiers sent");
                    return;
                }
                tracing::debug!("Render context not initialized; cannot show HWID notification");
            } else {
                tracing::debug!("Context lock busy; skipping HWID spoof notification");
            }
        } else {
            tracing::debug!("UI context unavailable; skipping HWID spoof notification");
        }
    }

    fn should_emit_notification(&self, now_ms: u64) -> bool {
        let last = self.last_notification_ms.load(Ordering::Relaxed);
        now_ms.saturating_sub(last) >= self.debounce_ms
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
