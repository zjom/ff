use std::path::PathBuf;

use crate::ast::Program;
use crate::interpreter::{Env, Scope, Value, eval_program};
use crate::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{ColorMode, Completer, EditMode, Editor, Helper, Highlighter, Hinter};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct REPLHelper {}

impl Validator for REPLHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        if input.is_empty() {
            return Ok(ValidationResult::Valid(None));
        }

        // If the input ends with a newline, it means the user pressed Enter on an empty line
        // at the end of a multi-line input, which we treat as a submission/abort.
        if input.ends_with('\n') {
            return Ok(ValidationResult::Valid(None));
        }

        match parse(input) {
            Ok(_) => {
                if ends_with_continuation_comma(input) {
                    Ok(ValidationResult::Incomplete)
                } else {
                    Ok(ValidationResult::Valid(None))
                }
            }
            Err(_) => {
                // Treat parse errors as incomplete to allow for multi-line input.
                Ok(ValidationResult::Incomplete)
            }
        }
    }
}

fn run_program(program: &Program, env: &Env) {
    match eval_program(program, env) {
        Ok(Value::Unit) => {}
        Ok(v) => println!("{}", v),
        Err(e) => eprintln!("error: {}", e),
    }
}

// Scan past whitespace, newlines, comments, and string literals to find the
// last syntactically meaningful character. Strings have no escape sequences
// in this grammar, so `"..."` matching is sufficient.
fn ends_with_continuation_comma(s: &str) -> bool {
    let mut last: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' => {
                while let Some(&n) = chars.peek() {
                    if n == '\n' || n == '\r' {
                        break;
                    }
                    chars.next();
                }
            }
            '"' => {
                last = Some('"');
                for c2 in chars.by_ref() {
                    if c2 == '"' {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => {}
            c => last = Some(c),
        }
    }
    last == Some(',')
}

#[derive(Debug, Clone, PartialEq, Eq, bon::Builder)]
pub struct Config {
    #[builder(default = EditMode::Vi)]
    edit_mode: EditMode,
    #[builder(default = ColorMode::Enabled)]
    color_mode: ColorMode,

    #[builder(default = ".ff_history", into)]
    history_path: PathBuf,
    #[builder(default = false)]
    should_write_history: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::builder().build()
    }
}

pub struct Repl {
    cfg: Config,
    rl: Editor<REPLHelper, FileHistory>,
}

impl Repl {
    pub fn new() -> Self {
        let cfg: Config = Default::default();
        Self::with_config(cfg).unwrap()
    }
    pub fn with_config(cfg: Config) -> anyhow::Result<Self> {
        let rlcfg = rustyline::Config::builder()
            .edit_mode(cfg.edit_mode)
            .color_mode(cfg.color_mode)
            .build();
        let helper = REPLHelper {};
        let mut rl = Editor::with_config(rlcfg)?;
        rl.set_helper(Some(helper));

        // Try to load history from a local file
        rl.load_history(&cfg.history_path)?;

        Ok(Self { cfg, rl })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let env = Scope::new();
        crate::prelude::install(&env);
        println!("ff self — Ctrl-D to exit, blank line to submit/abort multi-line input");
        loop {
            let prompt = ">> ";
            let readline = self.rl.readline(prompt);
            match readline {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    self.rl.add_history_entry(line.as_str()).ok();

                    match parse(&line) {
                        Ok(program) => {
                            run_program(&program, &env);
                        }
                        Err(e) => {
                            eprintln!("parse error: {}", e);
                        }
                    }
                    // Save history after each successful command
                    let _ = self.save_history();
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C: Clear the current buffer and start over
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    break Ok(());
                }

                Err(e) => {
                    eprintln!("error: {}", e);
                    Err(e)?;
                }
            }
        }
    }

    fn save_history(&mut self) -> anyhow::Result<()> {
        if self.cfg.should_write_history {
            self.rl.save_history(&self.cfg.history_path)?;
        }

        Ok(())
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<Config> for Repl {
    type Error = anyhow::Error;
    fn try_from(value: Config) -> Result<Self, Self::Error> {
        Self::with_config(value)
    }
}
