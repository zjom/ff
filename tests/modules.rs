mod common;

use common::{eval, eval_err};
use f2::eval::eval_file;
use std::path::{Path, PathBuf};

use std::sync::atomic::{AtomicU64, Ordering};

// ---------- temp-dir / fixture helpers ----------
//
// Each test gets a unique scratch directory. We avoid pulling in `tempfile`
// just for this — the path is `temp_dir()/ff_modules_test_<pid>_<n>/` and is
// removed at the end of the test (best-effort).

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TmpDir(PathBuf);

impl TmpDir {
    fn new() -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("ff_modules_test_{}_{}", std::process::id(), n));
        std::fs::create_dir_all(&path).expect("create tmp dir");
        TmpDir(path)
    }

    fn write(&self, rel: &str, body: &str) -> PathBuf {
        let full = self.0.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&full, body).expect("write fixture file");
        full
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_file(path: &Path) -> String {
    let v = eval_file(path)
        .unwrap_or_else(|e| panic!("eval_file failed: {}\n--- path ---\n{}", e, path.display()));
    format!("{}", v)
}

fn run_file_err(path: &Path) -> String {
    match eval_file(path) {
        Ok(v) => panic!(
            "expected eval_file error, got value {}\n--- path ---\n{}",
            v,
            path.display()
        ),
        Err(e) => e.to_string(),
    }
}

// ---------- native modules (no filesystem) ----------

#[test]
fn import_native_module_returns_object() {
    assert_eq!(eval(r#"typeof(import "Object")"#), ":object");
}

#[test]
fn import_native_module_exposes_named_members() {
    // `Object.get` is one of the documented members; calling it goes through
    // the same atom-key lookup that any imported module relies on.
    let src = r#"
        M = import "Object"
        M.get(:a, {:a: 1})
    "#;
    assert_eq!(eval(src), "1");
}

#[test]
fn bare_import_native_splats_members_into_scope() {
    // `import "Object"` as a bare statement should make `get`, `put`, `keys`,
    // `values` directly callable without the `Object.` prefix.
    let src = r#"
        import "Object"
        get(:b, {:a: 1, :b: 2})
    "#;
    assert_eq!(eval(src), "2");
}

#[test]
fn import_native_as_rhs_does_not_splat() {
    // When `import` is the RHS of `=`, it returns the module object but does
    // NOT pour its members into the surrounding scope.
    let src = r#"
        M = import "Object"
        get
    "#;
    // After the prelude is installed `get` exists on the env only if it was
    // splatted; otherwise it's an undefined-variable error. The stdlib does
    // not define a bare `get`, so this must error.
    let msg = eval_err(src);
    assert!(msg.contains("undefined"), "got: {}", msg);
}

#[test]
fn import_native_actor_module() {
    let src = r#"
        A = import "Actor"
        typeof(A.spawn)
    "#;
    assert_eq!(eval(src), ":native");
}

#[test]
fn import_native_fs_module() {
    let src = r#"
        F = import "Fs"
        typeof(F)
    "#;
    assert_eq!(eval(src), ":object");
}

#[test]
fn import_native_env_module() {
    let src = r#"
        E = import "Env"
        typeof(E)
    "#;
    assert_eq!(eval(src), ":object");
}

#[test]
fn import_path_must_be_string() {
    let msg = eval_err("import 42");
    assert!(
        msg.contains("String") || msg.contains("string"),
        "got: {}",
        msg
    );
}

#[test]
fn import_path_can_come_from_variable() {
    // `import <primary>` accepts any primary expression, including a bare
    // identifier — proves the path is evaluated, not just lexed.
    let src = r#"
        name = "Object"
        M = import name
        typeof(M)
    "#;
    assert_eq!(eval(src), ":object");
}

#[test]
fn import_unknown_module_file_read_error() {
    // Not a native module name and not a real file path on disk -> the
    // resolver falls through to fs read and surfaces a read failure.
    let msg = eval_err(r#"import "this_module_does_not_exist_xyz.ff""#);
    assert!(
        msg.contains("failed to read") || msg.contains("No such file"),
        "got: {}",
        msg
    );
}

// ---------- file-based modules ----------

#[test]
fn import_file_returns_named_exports() {
    let dir = TmpDir::new();
    dir.write(
        "math.ff",
        "square = x => x * x\ncube = x => x * x * x\nexport square, cube\n",
    );
    let main = dir.write(
        "main.ff",
        r#"M = import "math.ff"
[M.square(7), M.cube(3)]"#,
    );
    assert_eq!(run_file(&main), "[49, 27]");
}

#[test]
fn bare_import_file_splats_exports_into_scope() {
    let dir = TmpDir::new();
    dir.write("math.ff", "square = x => x * x\nexport square\n");
    let main = dir.write("main.ff", "import \"math.ff\"\nsquare(9)");
    assert_eq!(run_file(&main), "81");
}

#[test]
fn namespaced_import_does_not_splat() {
    // RHS-of-`=` import binds the module object but must not also splat into
    // scope, even when the module is a real file.
    let dir = TmpDir::new();
    dir.write("math.ff", "square = x => x * x\nexport square\n");
    let main = dir.write("main.ff", "M = import \"math.ff\"\nsquare(9)");
    let msg = run_file_err(&main);
    assert!(msg.contains("undefined"), "got: {}", msg);
}

#[test]
fn module_can_be_destructured_with_atom_keys() {
    // Modules are atom-keyed objects, so a pattern with explicit `:name` keys
    // destructures their exports directly.
    let dir = TmpDir::new();
    dir.write("lib.ff", "left = 1\nright = 2\nexport left, right\n");
    let main = dir.write(
        "main.ff",
        "{:left: l, :right: r} = import \"lib.ff\"\nl + r",
    );
    assert_eq!(run_file(&main), "3");
}

#[test]
fn unexported_bindings_are_not_in_module() {
    // `private` is defined in the module file but never exported, so the
    // imported object must not carry it.
    let dir = TmpDir::new();
    dir.write("lib.ff", "public = 1\nprivate = 2\nexport public\n");
    let main = dir.write("main.ff", "M = import \"lib.ff\"\nM.private");
    let msg = run_file_err(&main);
    assert!(msg.contains("no key"), "got: {}", msg);
}

#[test]
fn export_star_exports_all_top_level_bindings() {
    let dir = TmpDir::new();
    dir.write("lib.ff", "a = 1\nb = 2\nc = 3\nexport *\n");
    let main = dir.write("main.ff", "M = import \"lib.ff\"\n[M.a, M.b, M.c]");
    assert_eq!(run_file(&main), "[1, 2, 3]");
}

#[test]
fn export_undefined_name_is_runtime_error() {
    let dir = TmpDir::new();
    dir.write("lib.ff", "x = 1\nexport x, nope\n");
    let main = dir.write("main.ff", "import \"lib.ff\"\nx");
    let msg = run_file_err(&main);
    assert!(
        msg.contains("undefined") && msg.contains("nope"),
        "got: {}",
        msg
    );
}

#[test]
fn export_only_snapshots_value_at_time_of_export() {
    // `export` records the binding's current value. Rebinding the name
    // afterwards doesn't change what the module exposes.
    let dir = TmpDir::new();
    dir.write("lib.ff", "x = 1\nexport x\nx = 999\n");
    let main = dir.write("main.ff", "M = import \"lib.ff\"\nM.x");
    assert_eq!(run_file(&main), "1");
}

#[test]
fn repeated_export_keeps_latest_value() {
    // upsert_export updates an existing slot rather than appending a duplicate.
    let dir = TmpDir::new();
    dir.write("lib.ff", "x = 1\nexport x\nx = 2\nexport x\n");
    let main = dir.write("main.ff", "M = import \"lib.ff\"\nM.x");
    assert_eq!(run_file(&main), "2");
}

#[test]
fn export_operator_name_via_parens() {
    // `export (++)` — operator bindings can be exported by wrapping the name
    // in parens, same as defining them.
    let dir = TmpDir::new();
    dir.write("ops.ff", "(++) = (a, b) => a + b + 1\nexport (++)\n");
    let main = dir.write("main.ff", "import \"ops.ff\"\n2 ++ 3");
    assert_eq!(run_file(&main), "6");
}

#[test]
fn import_resolves_relative_to_importing_file() {
    // `lib/a.ff` imports `"b.ff"` — the resolver should look next to `a.ff`
    // (i.e. `lib/b.ff`), not next to `main.ff`.
    let dir = TmpDir::new();
    dir.write("lib/b.ff", "value = 42\nexport value\n");
    dir.write("lib/a.ff", "import \"b.ff\"\nexport value\n");
    let main = dir.write("main.ff", "M = import \"lib/a.ff\"\nM.value");
    assert_eq!(run_file(&main), "42");
}

#[test]
fn module_chains_imports() {
    // `a.ff` imports `b.ff` which imports `c.ff`. Each step re-exports its
    // dependency's name so the chain reaches `main.ff`.
    let dir = TmpDir::new();
    dir.write("c.ff", "leaf = \"hi\"\nexport leaf\n");
    dir.write("b.ff", "import \"c.ff\"\nexport leaf\n");
    dir.write("a.ff", "import \"b.ff\"\nexport leaf\n");
    let main = dir.write("main.ff", "M = import \"a.ff\"\nM.leaf");
    assert_eq!(run_file(&main), "\"hi\"");
}

#[test]
fn parse_error_in_module_surfaces_to_importer() {
    let dir = TmpDir::new();
    // Dangling operator — guaranteed parse error.
    dir.write("bad.ff", "x = 1 +\n");
    let main = dir.write("main.ff", "import \"bad.ff\"");
    let msg = run_file_err(&main);
    assert!(
        msg.contains("parse error") || msg.contains("expected"),
        "got: {}",
        msg
    );
}

#[test]
fn export_at_top_level_script_is_noop() {
    // A file run as a script (no enclosing importer) has no exports table.
    // The `export` statement should silently do nothing and the script's
    // final expression should still be returned.
    let dir = TmpDir::new();
    let main = dir.write("main.ff", "x = 7\nexport x\nx + 1\n");
    assert_eq!(run_file(&main), "8");
}

#[test]
fn import_inside_function_works() {
    // Imports aren't restricted to top level — they're just expressions.
    let dir = TmpDir::new();
    dir.write("math.ff", "square = x => x * x\nexport square\n");
    let main = dir.write(
        "main.ff",
        "load = () => import \"math.ff\"\nload().square(6)",
    );
    assert_eq!(run_file(&main), "36");
}

#[test]
fn module_does_not_leak_local_bindings_into_importer() {
    // Without `export`, a binding stays private. The importer's scope must
    // not gain access to `helper` just because the module file defines it.
    let dir = TmpDir::new();
    dir.write(
        "lib.ff",
        "helper = x => x * 2\nresult = helper(21)\nexport result\n",
    );
    let main = dir.write("main.ff", "import \"lib.ff\"\nhelper(10)");
    let msg = run_file_err(&main);
    assert!(msg.contains("undefined"), "got: {}", msg);
}

#[test]
fn module_imported_twice_returns_equal_values() {
    // Re-importing the same module yields a value the importer can use the
    // same way as the first import. (Whether the file is cached or
    // re-evaluated is an internal choice and not asserted here.)
    let dir = TmpDir::new();
    dir.write("lib.ff", "n = 7\nexport n\n");
    let main = dir.write(
        "main.ff",
        "A = import \"lib.ff\"\nB = import \"lib.ff\"\nA.n + B.n",
    );
    assert_eq!(run_file(&main), "14");
}

#[test]
fn imported_module_destructures_into_constituents() {
    assert_eq!(
        eval("{get, put} = import \"Object\"\nm = put :two 2 {:one: 1} \nget :two m "),
        "2"
    )
}
