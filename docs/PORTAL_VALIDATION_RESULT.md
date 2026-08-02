# Portal Backend Hosted Validation

**Trigger commit:** `fe2a6194c2ca6581bb073ebf3daf89f6ec9260de`  
**Run:** `30726772837`  
**Runner:** Ubuntu 22.04  
**Rust:** 1.88.0

- `cargo-format`: failure (exit 1)

## Sanitized failure output

```text
Diff in src-tauri/src/app.rs:98:
         CaptureBackendPreference::Auto => PortalScreenshotBackend::probe_area_support()
             .await
             .unwrap_or(false),
-        CaptureBackendPreference::Portal => {
-            PortalScreenshotBackend::probe_area_support().await?
-        }
+        CaptureBackendPreference::Portal => PortalScreenshotBackend::probe_area_support().await?,
     };
     match choose_backend_id(
         preference,
Diff in src-tauri/src/app.rs:125:
     match preference {
         CaptureBackendPreference::Portal if portal_supported => Ok(CaptureBackendId::XdgPortal),
         CaptureBackendPreference::Portal => Err(AppError::CaptureBackendUnavailable),
-        CaptureBackendPreference::Gnome if gnome_available => {
-            Ok(CaptureBackendId::GnomeScreenshot)
-        }
+        CaptureBackendPreference::Gnome if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
         CaptureBackendPreference::Gnome => Err(AppError::CaptureBackendUnavailable),
         CaptureBackendPreference::Auto if portal_supported => Ok(CaptureBackendId::XdgPortal),
-        CaptureBackendPreference::Auto if gnome_available => {
-            Ok(CaptureBackendId::GnomeScreenshot)
-        }
+        CaptureBackendPreference::Auto if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
         CaptureBackendPreference::Auto => Err(AppError::CaptureBackendUnavailable),
     }
 }
Diff in src-tauri/src/app.rs:153:
     #[test]
     fn automatic_selection_prefers_proven_portal_area_capture() {
         assert_eq!(
-            choose_backend_id(CaptureBackendPreference::Auto, true, true)
-                .expect("portal backend"),
+            choose_backend_id(CaptureBackendPreference::Auto, true, true).expect("portal backend"),
             CaptureBackendId::XdgPortal
         );
     }
Diff in src-tauri/src/app.rs:162:
     #[test]
     fn automatic_selection_falls_back_before_opening_a_selector() {
         assert_eq!(
-            choose_backend_id(CaptureBackendPreference::Auto, false, true)
-                .expect("GNOME backend"),
+            choose_backend_id(CaptureBackendPreference::Auto, false, true).expect("GNOME backend"),
             CaptureBackendId::GnomeScreenshot
         );
     }
Diff in src-tauri/src/capture/portal.rs:3:
 use crate::image_pipeline::decode_captured_image;
 use crate::models::{CaptureBackendId, CapturedImage};
 use ashpd::desktop::{
-    ResponseError,
     screenshot::{AvailableTargets, Screenshot, ScreenshotProxy},
+    ResponseError,
 };
 use ashpd::{Error as PortalClientError, PortalError};
 use async_trait::async_trait;
Diff in src-tauri/src/capture/portal.rs:37:
             .await
             .map_err(|_| AppError::CaptureBackendUnavailable)?
             .map_err(map_portal_error)?;
-        Ok(supports_area(version, targets.contains(AvailableTargets::Area)))
+        Ok(supports_area(
+            version,
+            targets.contains(AvailableTargets::Area),
+        ))
     }
 
     async fn capture_area(&self) -> Result<CapturedImage, AppError> {
Diff in src-tauri/src/capture/portal.rs:72:
     match error {
         PortalClientError::Response(ResponseError::Cancelled)
         | PortalClientError::Portal(PortalError::Cancelled(_)) => AppError::CaptureCancelled,
-        PortalClientError::Portal(PortalError::NotAllowed(_)) => {
-            AppError::CapturePermissionDenied
-        }
+        PortalClientError::Portal(PortalError::NotAllowed(_)) => AppError::CapturePermissionDenied,
         PortalClientError::RequiresVersion(_, _) | PortalClientError::PortalNotFound(_) => {
             AppError::CaptureBackendUnavailable
         }

```

**Result:** failure at `cargo-format`
