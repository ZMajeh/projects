# Tutorial: Developing with Rust & Leptos

This tutorial guides you through writing frontend web applications using **Rust** and the **Leptos Framework**. It assumes zero prior Rust frontend experience and will walk you through foundational Rust syntax, Leptos reactivity concepts, and the structure of our Lodge Management application.

---

## 1. Rust Foundations for Frontend Developers

If you are coming from JavaScript, TypeScript, or React, some Rust concepts require initial adjustment. Below are the key language features used extensively in Leptos:

### Variables and Mutability
By default, variables in Rust are immutable. You must use `mut` to make them mutable:
```rust
let name = "Amit"; // Immutable
let mut count = 0; // Mutable
count += 1;
```

### Options (`Option<T>`)
Rust does not have a null/undefined value. Instead, it uses the `Option<T>` enum to express when a value might be absent:
```rust
let user_name: Option<String> = Some("Amit Kumar".to_string());
let logged_out_user: Option<String> = None;

// Extracting values
match user_name {
    Some(name) => println!("Logged in as {}", name),
    None => println!("Anonymous user"),
}
```

### Results (`Result<T, E>`)
Rust handles potential failures using the `Result<T, E>` enum, which contains either `Ok(value)` or `Err(error)`:
```rust
fn divide(a: f32, b: f32) -> Result<f32, &'static str> {
    if b == 0.0 {
        Err("Cannot divide by zero!")
    } else {
        Ok(a / b)
    }
}
```

---

## 2. Leptos Core Concepts

Leptos is a high-performance declarative framework. Here is how it compares to React:

| Concept | React (JS/TS) | Leptos (Rust) |
|---|---|---|
| **Compilation** | Transpiled to JS | Compiled to WebAssembly |
| **Reactivity** | Virtual DOM (re-renders component) | Fine-grained signals (direct DOM updates) |
| **State** | `useState(initial)` | `create_signal(initial)` |
| **Markup** | JSX (`<div />`) | `view! { <div> </div> }` macro |

### Signals: The Primitive of Reactivity
A signal consists of a **ReadSignal** and a **WriteSignal**. Reading a signal subscribes the caller (like a DOM node) to future updates:
```rust
use leptos::*;

#[component]
pub fn Counter() -> impl IntoView {
    // create_signal returns (ReadSignal, WriteSignal)
    let (count, set_count) = create_signal(0);

    view! {
        <div>
            <p>"Current count: " {count}</p>
            // Note: closures 'move' variables to capture state
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                "Increment"
            </button>
        </div>
    }
}
```

*Note: Why use closures `move ||` inside the view? In Leptos, a non-reactive dynamic value is rendered once. Passing a closure `move || count.get()` (or shorthand `{count}`) makes that specific DOM text node reactive. When `set_count` triggers, ONLY that text node is updated in the browser!*

---

## 3. Creating Components

Leptos components are written as standard Rust functions decorated with the `#[component]` macro. Every component must return `impl IntoView`.

### Rules of Components:
1. **Component Name**: Must be in PascalCase.
2. **Return Type**: Always `impl IntoView`.
3. **Props**: Defined as arguments to the function. Leptos automatically derives prop-building builders.
4. **Closing Tags**: Elements inside the `view!` macro must close properly.

### Example: A Reusable Aadhaar Input Component
Here is an example demonstrating custom component creation, props, validation, and passing callbacks back to parent layouts:

```rust
use leptos::*;

#[component]
pub fn AadhaarInput(
    // Receives a read-only string signal from parent
    value: ReadSignal<String>,
    // Callback to emit value updates to parent
    on_change: Callback<String>,
) -> impl IntoView {
    
    // Local signal for validation errors
    let (error, set_error) = create_signal(Option::<String>::None);

    let handle_input = move |ev| {
        let val = event_target_value(&ev);
        
        // Aadhaar validation: check if exactly 12 digits
        if val.chars().all(|c| c.is_ascii_digit()) && val.len() == 12 {
            set_error.set(None);
            on_change.call(val);
        } else {
            set_error.set(Some("Aadhaar must be exactly 12 numeric digits.".to_string()));
            on_change.call("".to_string()); // clear valid value in parent
        }
    };

    view! {
        <div class="form-group">
            <label>"Aadhaar Number"</label>
            <input 
                type="text" 
                value=value
                on:input=handle_input
                placeholder="1234 5678 9012"
                maxlength="12"
                class="form-control"
            />
            {move || error.get().map(|err| view! {
                <span class="text-error">{err}</span>
            })}
        </div>
    }
}
```

---

## 4. State Communication & Layout Hierarchies

State travels downwards as signals, while changes travel upwards via event closures or Callbacks.

### Parent-to-Child Communication
To pass reactive signals to children, you can pass:
1. **The ReadSignal itself**: `ReadSignal<T>` allowing the child to subscribe directly.
2. **A closure mapping value**: `move || signal.get()`.
3. **`MaybeSignal<T>`**: A type-safe envelope that accepts either a static `T` or a reactive `Signal<T>`.

### Child-to-Parent Communication
To update the parent state from a child, pass:
1. **A `WriteSignal<T>`**: Child can directly mutate the value (e.g., `set_count.set(...)`).
2. **An explicit callback**: `Callback<Input, Output>` (as seen in `AadhaarInput` above).

### App Orchestration Example:
```rust
#[component]
pub fn LodgeDashboard() -> impl IntoView {
    let (selected_room, set_selected_room) = create_signal("Room 101".to_string());
    
    let on_room_changed = Callback::new(move |new_room| {
        set_selected_room.set(new_room);
    });

    view! {
        <div class="dashboard-container">
            <h1>"Current Selection: " {selected_room}</h1>
            
            // Sub-component invocation:
            <RoomSelector 
                current_selection=selected_room 
                on_select=on_room_changed 
            />
        </div>
    }
}
```
This elegant structure provides deterministic data flow: state is held in the orchestrator component and cleanly propagated to subviews.
