# Technology: Vanilla HTML & CSS

The visual layer is designed using custom responsive HTML structures and highly modular CSS styles.

## How It Helps the Lodge Management System
* **Visual Polish & Branding**: Establishes a modern, clean, and intuitive color-coded UI (blue for primaries, gold/amber for warnings/pending rooms, green for available, red for occupied).
* **Zero Performance Overhead**: Avoids heavy utility classes or bloated CSS frameworks (like Tailwind), keeping compiling times fast and bundle sizes lightweight.
* **Print-Friendly Designs**: Provides special CSS rules (`@media print`) so that invoices, bills, and check-out logs print in clean, professional tabular formats perfectly optimized for physically printable documents.

## Specific Role in this Project
* Defines custom properties (`:root`) for easy theme modification.
* Sets up the dashboard layouts, cards, buttons, responsive media grids, and overlays.
* Controls typography, component spacing, and interactive transitions.
