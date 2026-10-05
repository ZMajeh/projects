# Technology: Tesseract.js (Optical Character Recognition)

**Tesseract.js** is a pure JavaScript port of the popular Tesseract OCR engine, running directly in the browser via Web Workers.

## How It Helps the Lodge Management System
* **Local, Secure Processing**: Capturing Aadhaar details requires text extraction. Processing OCR locally in-browser saves backend costs, is faster, and improves customer privacy as sensitive data does not need to be sent to external server APIs for recognition.
* **Smooth Check-in Workflows**: Instantly translates a scanned picture of an Aadhaar card into structured text (Aadhaar number, Name, DOB, Address), letting the operator review and auto-populate the record with a single click.

## Specific Role in this Project
Exposed via the JS Bridge as `window.extractAadhaar`, it is called when a customer's document is scanned or selected, feeding parsed textual data back to the Rust customer controller.
