# Play Console — Data safety and photo access

Use this when filling the Google Play Data safety form and the Photo and Video Permissions declaration. It matches the app as shipped.

## Data safety

| Question | Answer |
|---|---|
| Does the app collect or share user data? | No data is sent to a MooJot server. There is no analytics SDK. |
| Is data encrypted in transit? | Not applicable for the default on-device ledger. If the user saves an AksonOCR API key, that optional request uses HTTPS. |
| Can users ask for deletion? | They delete records in the app, or uninstall. Data is not held off-device. |
| Financial info | Entered or imported by the user and stored only on the device. Not shared. |
| Photos | Read to import Thai bank slips. Stored as private copies inside the app. Not shared. |

Declare **Photos and videos** as collected for app functionality, not shared, processed on device.

If the optional AksonOCR key stays in the product, also declare that photos may be sent to the OCR provider **only when the user supplies a key**. The default build path does not.

## Photo and video permissions

- Permission: `READ_MEDIA_IMAGES` (and `READ_MEDIA_VISUAL_USER_SELECTED` on Android 14+).
- Core use: scan the user's own Thai bank-slip albums.
- Alternative: the in-app picker uses `ACTION_GET_CONTENT`, so a user can import one slip or one PDF without granting album access.
- `Camera` and `Screenshots` folders are not scanned. Shared albums are off unless the user opts in, and a slip QR is required before OCR.

## Listing assets

- App icon: `src-tauri/gen/android/app/src/main/res/mipmap-*/`
- Feature graphic source: [play-feature-graphic.svg](play-feature-graphic.svg) (1024×500)
- Screenshots: capture 4–6 phone screens in Pink & White and Black & Orange (Home, empty month, Summary, My Moo) on a device. Do not invent them.

## Not in this pipeline

Biometric app lock and a Jetpack Glance home-screen widget were listed as optional. They are not included here.
