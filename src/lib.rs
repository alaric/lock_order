use proc_macro::{self, TokenStream};

#[derive(Clone, PartialEq, Debug, Default)]
struct LockItem {
    last_identifier: String,
    full_identifier: String,
    mutable: bool,
}

impl LockItem {
    fn add(&mut self, id: &proc_macro::TokenTree) {
        self.full_identifier += &id.to_string();
        self.last_identifier = id.to_string();
    }
}

#[proc_macro]
pub fn lock(item: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    let mut curr = LockItem::default();
    for i in item {
        match i.to_string().as_str() {
            "mut" => {
                curr.mutable = true;
            }
            "," => {
                out.push(curr);
                curr = LockItem::default();
            }
            _ => {
                curr.add(&i);
            }
        }
    }

    if curr != LockItem::default() {
        out.push(curr);
    }

    out.sort_by(|a, b| a.last_identifier.partial_cmp(&b.last_identifier).unwrap());

    let declarations: Vec<String> = out
        .clone()
        .into_iter()
        .map(|x| {
            if x.mutable {
                format!("mut {}", x.last_identifier)
            } else {
                x.last_identifier.clone()
            }
        })
        .collect();
    let locks: Vec<String> = out
        .into_iter()
        .map(|x| format!("{}.lock().unwrap()", x.full_identifier))
        .collect();

    format!(
        "let ({}) = ({});",
        declarations.join(", "),
        locks.join(", "),
    )
    .parse()
    .unwrap()
}
