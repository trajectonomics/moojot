# MooJot Privacy Policy

**Last updated: 22 November 2026**

Trajectonomics publishes MooJot, a personal income and expense tracker. This policy says what the Android app does with information. The same text is shown in My Moo → About → Privacy.

The short version: the ledger stays on your phone. Trajectonomics has no account server and does not receive your records. To read English text on slips, the app uses Google ML Kit. ML Kit reads the photo on the phone, but it sends Google metrics about its own performance and use, such as your device model and a per-installation identifier. MooJot cannot switch that off. The section "Google ML Kit metrics" below lists what is sent. The only upload of a slip image the code can make is optional. It happens only if an AksonOCR API key is already stored in the local database. The settings screen does not ask for that key.

This policy is written so a person can read it. It is the project's description of its own behavior. It is not a certificate of compliance, and it is not legal advice.

## Who we are

The copyright notice on MooJot names Trajectonomics. The repository does not list a registered office. For this policy, "we" means the MooJot project published by Trajectonomics at <https://github.com/trajectonomics/moojot>.

Because the ledger never reaches us, we cannot see it, correct it, or delete it from our side. You hold it on your phone.

## Information that stays on the phone

MooJot stores the following in the app's private files. None of it is sent to a MooJot server.

| Data | Where | Why |
| --- | --- | --- |
| Income and expense records: amount in satang, currency, date, time, account, category, payee, note, tags, source | SQLite on the phone | Your ledger |
| Budgets, recurring rules, categories, tags, and the bank list you enable | SQLite on the phone | Categories, limits, and which albums to scan |
| Slip photo, QR contents, recognized text, and a hash of the image | App private storage and SQLite | Attach the slip you imported |
| Text taken from a credit-card statement PDF | SQLite, as records | Import a statement you chose |
| PDF password | Phone memory for that import only | Open the file. It is not written to the database. |
| Owner names you type, language, month-start day, week start, shared-album switch | SQLite on the phone | Tell your own transfers apart, and set the calendar |
| AksonOCR API key and model name, if a key is present | SQLite on the phone, as plain text | Optional cloud OCR. The key is not encrypted by a separate MooJot secret. |
| Ids of photos already scanned, and months already scanned | SQLite on the phone | Do not repeat work |
| Daily counts of slips synced and records categorized | SQLite on the phone | A local activity count, not a remote log |
| Theme (pink or dark) and hand (left or right) | localStorage on the phone | Remember appearance. These two keys are not tracking cookies. |

Slip photos and statement lines can contain names, account details, amounts, and other information that is sensitive under Thailand's Personal Data Protection Act (PDPA). That sensitivity is why it stays in the app's private storage unless you export it or turn on the optional OCR key described below.

## How information gets there

You type a record, you scan a bank album, you pick one photo, or you pick a statement PDF. The app can also read albums you enable, using Android photo access (`READ_MEDIA_IMAGES`, and `READ_MEDIA_VISUAL_USER_SELECTED` on Android 14 and newer). On older Android versions the manifest also declares `READ_EXTERNAL_STORAGE`, limited to those old versions.

You can import one slip or one PDF with the system picker instead of granting album access. Shared albums stay off until you turn them on. The scanner looks for a slip QR before it treats a photo as a slip. Camera and Screenshots folders are not the scan target.

On-device tools read the slip locally: a QR decoder, ML Kit Latin for English text, and AksonOCR-Lite (an ONNX model) for Thai text. Those steps do not send the photo to Trajectonomics or to Google as part of MooJot's design. ML Kit does send Google metrics about how it runs, described in the next section.

The manifest declares `INTERNET` so ML Kit can send those metrics and the optional OCR request below can use HTTPS. The manifest also declares `POST_NOTIFICATIONS`. The current app does not send notification content anywhere and does not use notifications to collect data.

## Google ML Kit metrics

MooJot uses Google ML Kit text recognition to read English text on a slip. ML Kit is a library made by Google. It is not open source, and Google's ML Kit Terms of Service apply to it: <https://developers.google.com/ml-kit/terms>

ML Kit reads the photo on the phone. Google states that ML Kit does not send the image, or the text it reads, to Google servers. It does send Google metrics about the performance and use of ML Kit in the app. Google's ML Kit data disclosure (<https://developers.google.com/ml-kit/android-data-disclosure>) lists these kinds of data:

- device information, such as the manufacturer, model, Android version and build
- app information: the package name and app version
- a per-installation identifier, which Google says is not intended to identify a user or a physical device
- performance metrics, such as how long recognition took
- the API configuration, such as the image format and resolution, and the size of the input and output
- the ML Kit feature version, the type of event (such as start-up, a recognition run, or a release), and error codes

Google says ML Kit sends this over HTTPS, uses it for diagnostics and usage analytics, and does not transfer it to third parties. Google's Privacy Policy (<https://policies.google.com/privacy>) applies to it. Trajectonomics does not receive these metrics and cannot delete them.

MooJot has no setting that switches these metrics off, and Google's documentation does not describe one. ML Kit's text reader runs when you scan bank albums, pick a slip photo, share a slip to MooJot, or re-parse slips. Typing a record by hand does not use it.

## Optional AksonOCR upload

If, and only if, the local settings contain a non-empty AksonOCR API key, and a slip still needs Thai text, the app can send that slip image over HTTPS to AksonOCR at `https://backend.aksonocr.com/api/v2/ocr`. The image can contain the personal and financial details printed on the slip. The API key is sent with that request.

We do not operate AksonOCR. We do not get a copy of the image back onto a server of ours. What AksonOCR stores, and for how long, is governed by AksonOCR, not by this policy. Leave the key empty and this request is not made. A new install has an empty key.

The My Moo screen that describes local OCR is about the on-device model. It does not switch this optional path off if a key is already in the database.

## What we do not collect

We do not collect:

- a MooJot account, email, or password
- a payment card for Trajectonomics (the app is free and we take no payment)
- advertising identifiers, analytics events, or crash reports
- your contacts, location, or microphone
- the ledger, by any automatic upload

Google receives the ML Kit metrics described above. We do not.

We do not sell personal information. We do not share it for cross-context advertising. We do not train a model on your records.

## When information leaves the phone

Information leaves the phone only in these cases:

- You export CSV or JSON, or you use the Android share sheet. The file goes to the app you pick. We do not upload it.
- ML Kit reads a slip, and sends Google the metrics described in "Google ML Kit metrics". This does not include the photo or the text read from it.
- You stored an AksonOCR API key, and the app sends one slip image to AksonOCR as described above.
- You copy text into a GitHub issue or a security report yourself. Do not put real account numbers, slip photos, or a full ledger in a public issue.

Banks, Google (including Google Play and ML Kit), Android, and AksonOCR are third parties. Their own terms apply when you use them. We are not responsible for their outages or their privacy practices.

## How long it stays

Records stay on the phone until you delete them, clear the app's storage, or uninstall MooJot. Android normally deletes the app's private storage on uninstall. A backup you made, or a file you exported, is outside the app and stays until you delete that copy.

We have no server copy, so we have no server retention period. We also have no server backup we could restore for you.

## Security

The database sits in the app's private storage, which other apps are not supposed to read. That is not the same as a promise of encryption at rest. MooJot does not add a separate passphrase or biometric lock in this version. Your phone lock is your lock.

No method of storage is perfectly safe. Someone who unlocks your phone, or who has a backup of it, may be able to read the database. If you save an AksonOCR key, it is stored in that database as plain text.

The optional OCR call uses HTTPS. A PDF password is used in memory to decrypt the file and is not saved.

## Your choices

On the phone you can:

- edit or delete a record
- turn banks and shared albums on or off
- refuse photo access and use the picker instead
- export a copy
- uninstall the app and remove its private storage

Those are the practical ways to access, correct, and delete the ledger. We cannot do those steps for you, because we never receive the data.

### Thailand (PDPA)

If the PDPA applies to you, the personal data in MooJot is processed on your device for your own household ledger. Trajectonomics does not collect that ledger onto its own system. For the optional AksonOCR send, you cause the transfer by having a key stored, and AksonOCR is the recipient. ML Kit metrics go to Google, under Google's Privacy Policy.

You may ask questions about this policy through the contact links below. A request to delete data we do not hold can only be answered by pointing you back to the app, the uninstall step, and, if you used it, AksonOCR.

### European Economic Area and the United Kingdom

If you use the app from the EEA or the UK, the same facts apply. We do not operate a server that holds your ledger, so we are not in a position to act as a remote controller of that ledger. The legal basis for any optional image you send to AksonOCR is the step you take by storing a key and scanning. You can stop that path by clearing the key or leaving it empty. Rights of access, correction, deletion, and portability for the ledger are exercised on the device, including by export and uninstall.

We do not transfer your ledger outside your phone. ML Kit metrics go to Google, and an optional AksonOCR request goes to that provider. Each may process the data in whatever country it uses, under its own safeguards.

### California

We do not sell personal information and we do not share it for cross-context behavioral advertising. We do not offer a financial incentive for personal information. Categories that can exist on the device are identifiers inside a slip (such as a name or account reference), financial information you enter, photos you import, and the local settings listed above. We do not collect those categories onto a MooJot server. Google receives ML Kit metrics, including a per-installation identifier, as described above.

## Children

MooJot is not directed at children under 13, and we do not knowingly collect personal information from children on a server. The app has no account that a child could register with us. If a child uses a phone where the app is installed, the records remain on that phone.

## Changes to this policy

We will change the "last updated" date when this policy changes, and we will put the new text in the app's About screen and in `docs/privacy-policy.md` on GitHub. Please read it when the date changes. Continuing to use the app after the new date means you accept the update.

## Contact

Questions about this policy, and privacy requests we are actually able to answer, go to:

- <https://github.com/trajectonomics/moojot/issues>
- Security: <https://github.com/trajectonomics/moojot/security/advisories/new>

There is no privacy email and no postal address published for this project. Please do not include real bank details, slip images, or an API key in a public issue.
