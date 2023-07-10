# The Goal

Turn this:

```rust
struct Counter {
    count: usize,
}

impl Counter {
    pub fn handle_button_click(&mut self, event: Event) {
        self.count += 1;
    }

    pub fn render(&self) -> Option<DomNode> {
        html! {
            <p>The current count is {self.count}</p>
            <button on:click={self.handle_button_click}>Click to count!</button>
        }
    }
}
```

Into this:

```rust
impl CounterFull {
    pub fn handle_event(&mut self, event: Event, element: usize) {
        match (event, element) {
            (Event::Click, 1) => {
                self.count += 1;
            }
            _ => (),
        }
    }

    pub fn render(&self) -> Option<DomNode> {
        Some(&[
            DomNode::p().text_content(format!("The current count is {}", self.count)),
            DomNode::button().text_content("Click to count!"),
        ])
    }
}
```

# Requirements

 - [ ] Macros
     - [ ] Macro to parse HTML-like syntax into Rust builder pattern
     - [ ] Proc macro to create `handle_event` method
 - [ ] Way to create DOM elements from Rust
 - [ ] Way to send events to Rust

