# Tutorial: Tesseract.js (OCR)

Using Tesseract.js for in-browser OCR.

## Setup
Ensure the Tesseract library is included in `index.html`.

## Usage
The JS bridge defines `extractAadhaar`.

```javascript
// In index.html JS Bridge
async function extractAadhaar(imageBase64) {
    const worker = await Tesseract.createWorker();
    const { data: { text } } = await worker.recognize(imageBase64);
    await worker.terminate();
    return text;
}
```

## Rust Calling
```rust
// src/api.rs
#[wasm_bindgen]
extern "C" {
    pub async fn extractAadhaar(base64: String) -> JsValue;
}
```
