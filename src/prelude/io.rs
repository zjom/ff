use crate::interpreter::Value;
use crate::native;
use std::io::Write;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        (
            "println",
            native!("io.println", 1, |ctx, args| {
                let s = render(&args[0]);
                writeln!(ctx.out.borrow_mut(), "{}", s)
                    .map_err(|e| anyhow::anyhow!("io error: {}", e))?;
                Ok(Value::Unit)
            }),
        ),
        (
            "print",
            native!("io.print", 1, |ctx, args| {
                let s = render(&args[0]);
                write!(ctx.out.borrow_mut(), "{}", s)
                    .map_err(|e| anyhow::anyhow!("io error: {}", e))?;
                Ok(Value::Unit)
            }),
        ),
    ]
}

// Strings print without quotes; everything else uses the language's Display.
fn render(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        v => v.to_string(),
    }
}
