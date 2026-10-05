# Majeh's PDF Viewer (Touch Support Edition) 🚀

## 🌟 What makes this Viewer special?
Majeh's PDF Viewer is not like other heavy PDF readers. It is designed to be **super-lightweight** and **lightning-fast**. It doesn't have hundreds of confusing buttons; instead, it focuses on giving you the cleanest and fastest reading experience possible.

This version is uniquely improved with **Native Touch Support**, meaning if you have a laptop or monitor with a touch screen, you can pinch your fingers to zoom in and out perfectly—just like you do on your smartphone or tablet!

### **⚡ Performance Stats:**
*   **Launch Time:** Near-instant (Opens in less than 0.5 seconds).
*   **Space Usage:** Only **~10 MB** (Most other readers are 100MB to 500MB!).
*   **Memory (RAM):** Very low (Uses about 30MB to 50MB of RAM while reading standard books).

### **📁 What can it open?**
*   **Documents:** PDF, XPS, OpenXPS, EPUB, FB2.
*   **Comic Books:** CBZ.
*   **Images:** PNG, JPG, JPEG, TIFF, BMP, GIF.

## 🎮 How to Use It
*   **Zooming:** Use two fingers to pinch the screen, or press `+` and `-` on your keyboard.
*   **Mouse Zoom:** Hold **Ctrl** and scroll your mouse wheel.
*   **Moving:** Use the **Arrow Keys** to move around the page.
*   **Whole Page:** Press `Z` to see the whole page at once.
*   **Full Screen:** Press `F` to hide everything except the book.
*   **Rotate:** Press `R` if the book is sideways.

---

## 📂 The 8 Parts of this Project (Binaries)

A "binary" is just a file that your computer can run (an `.exe` file). This project makes 8 of them:

### 1. majehs-viewer.exe (The Main App)
**What it is:** This is the window you use to read your books and pictures.
**How the code works:**
```
[ Open Program ]
      |
[ Look at the File ]
      |
[ Draw the Page ] <-----------+
      |                       |
[ Wait for You ] -------------+
(Did you pinch? Click? Press a key?)
```
*   **Main Task (`viewproc`):** It watches for your mouse, fingers, or keyboard.
*   **Drawing Task (`winblit`):** It paints the words and pictures on your screen.
*   **How to use:** Just double-click it and pick a file, or drag a file onto it.
*   **Example:** `majehs-viewer.exe my_book.pdf`
*   **Hidden Detail:** If you press the `m` key, it "bookmarks" your page. Press `t` to jump back to it later!

---

### 2. mutool.exe (The Toolbox)
**What it is:** A "Swiss Army Knife" for PDFs. It can fix broken files or tell you secrets about a document.
**How the code works:**
```
[ Start ] -> [ Pick a Tool ] -> [ Run that Tool ]
```
*   **How to use:** Open a terminal and type `mutool` followed by a command:
    *   `info`: Shows how many pages and what fonts are inside.
    *   `clean`: Fixes a PDF that won't open.
    *   `extract`: Takes all the pictures out of a PDF and saves them.
*   **Example:** `mutool info document.pdf`
*   **Hidden Detail:** If you rename this file to `mutoolinfo.exe`, it will always run the "info" command automatically!

---

### 3. mudraw.exe (The Picture Maker)
**What it is:** This tool turns document pages into high-quality pictures (like PNG or JPG).
**How the code works:**
```
[ Read PDF ] -> [ Draw Page in Memory ] -> [ Save as Picture File ]
```
*   **How to use:** `mudraw -o output.png input.pdf`
*   **Example:** `mudraw -o page1.png my_book.pdf 1` (This saves page 1 as a picture).
*   **Hidden Detail:** Use the `-i` flag to turn the page colors "upside down" (Dark Mode).

---

### 4. mujstest.exe (The Robot Brain Tester)
**What it is:** Some PDFs have "brains" (JavaScript) to handle forms or buttons. This tool tests if those brains are working correctly.

---

### 5. The "Helper" Tools (Construction Workers)
These tools are used by the computer to build the final app:
*   **cmapdump.exe:** Helps the app understand different languages (like Chinese or Japanese).
*   **fontdump.exe:** Puts the fonts inside the app so it works on any computer.
*   **bin2hex.exe:** Turns files into code that the app can read easily.
*   **cquote.exe:** A small tool to help handle text for the computer.

---

## 🛠 How to Make (Build) the App

### For Windows:
1.  **Get the tools:** Run `.\scripts\install_deps.ps1` as an Admin (only needs to be done once).
2.  **Make it:** Run `.\build.ps1`.
3.  **Find it:** Your new app will be in `build/release/majehs-viewer.exe`.

### For Windows File Icons:
*   In the `build/release` folder, **Right-click `register.bat`** and choose **Run as Administrator**. Now all your documents and images will show Majeh's cool icon!

---

## 📝 License
Built with love by Majeh, using the MuPDF engine (Copyright 2006-2014 Artifex Software, Inc.).
