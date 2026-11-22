# Play Console — Data safety and photo access

Use this when filling the Google Play Data safety form and the Photo and Video Permissions declaration. It matches the app as shipped.

## Data safety

| Question | Answer |
|---|---|
| Does the app collect or share user data? | No data is sent to a MooJot server. The bundled Google ML Kit SDK sends Google usage and performance metrics (see below), and Play counts data an SDK sends as collected by the app. |
| Is data encrypted in transit? | Yes. ML Kit sends its metrics over HTTPS. If the user saves an AksonOCR API key, that optional request also uses HTTPS. |
| Can users ask for deletion? | They delete records in the app, or uninstall. The ledger is not held off-device. ML Kit metrics are held by Google, and MooJot cannot delete them. |
| Financial info | Entered or imported by the user and stored only on the device. Not shared. |
| Photos | Read to import Thai bank slips. Stored as private copies inside the app. Not shared. |

Declare **Photos and videos** as collected for app functionality, not shared, processed on device.

### Google ML Kit metrics

ML Kit (English slip text) sends Google metrics whenever it reads a slip. Google's disclosure is <https://developers.google.com/ml-kit/android-data-disclosure>, and it leaves the form answers to the developer. Per that page:

- What is sent: device information (manufacturer, model, OS version and build), app information (package name and version), a per-installation identifier, performance metrics, the API configuration (such as image format and resolution), input and output size, feature version, event type, and error codes. Not the photo or the recognized text.
- Purpose, in Google's words: diagnostics and usage analytics.
- Encrypted in transit (HTTPS). Google says it does not transfer this data to third parties.
- Users cannot turn it off in the app, so mark it as required collection.

In the form this most closely matches **App info and performance** (diagnostics, other performance data) and **Device or other IDs**, collected for **Analytics**. Check the current Play data-type definitions when you fill it in.

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
