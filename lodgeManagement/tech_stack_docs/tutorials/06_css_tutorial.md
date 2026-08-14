# Tutorial: Vanilla CSS Styling

We use Vanilla CSS for styling to keep the application lightweight.

## Structure
- Global styles defined in `:root` in `index.html`.
- Component-specific styles can be added to a shared CSS file or directly in the `<style>` block of `index.html`.

## Printing
Use `@media print` for printable bills:
```css
@media print {
    .no-print { display: none; }
    .printable-table { width: 100%; border: 1px solid black; }
}
```
