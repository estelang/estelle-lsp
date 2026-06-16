pub const KEYWORDS: &[&str] = &[
    "fnc", "pub", "import", "as", "return", "output", "and", "or", "not", "if", "else", "for",
    "in", "while", "repeat", "break", "continue", "try", "catch", "lua", "true", "false", "nil",
    "str", "num", "bool", "list", "map",
];

pub struct BuiltinDoc {
    pub name: &'static str,
    pub signature: &'static str,
    pub doc: &'static str,
}

pub const BUILTINS: &[BuiltinDoc] = &[
    BuiltinDoc {
        name: "trim",
        signature: "trim(s str) str",
        doc: "Strip leading/trailing whitespace.",
    },
    BuiltinDoc {
        name: "lower",
        signature: "lower(s str) str",
        doc: "Lowercase (Unicode-aware).",
    },
    BuiltinDoc {
        name: "upper",
        signature: "upper(s str) str",
        doc: "Uppercase (Unicode-aware).",
    },
    BuiltinDoc {
        name: "len",
        signature: "len(s str|list) num",
        doc: "Length. Str → Unicode codepoints; list → count.",
    },
    BuiltinDoc {
        name: "sub",
        signature: "sub(s str, i num, j num) str",
        doc: "Substring (1-based, inclusive).",
    },
    BuiltinDoc {
        name: "find",
        signature: "find(s str, pat str)",
        doc: "Pattern find (mw.ustring.find).",
    },
    BuiltinDoc {
        name: "replace",
        signature: "replace(s str, pat str, rep str) str",
        doc: "Pattern replace. Multi-return: wrap in pipes with care.",
    },
    BuiltinDoc {
        name: "split",
        signature: "split(s str, sep str) list",
        doc: "Split string into list.",
    },
    BuiltinDoc {
        name: "join",
        signature: "join(list list, sep str) str",
        doc: "Join list into string.",
    },
    BuiltinDoc {
        name: "floor",
        signature: "floor(n num) num",
        doc: "Floor.",
    },
    BuiltinDoc {
        name: "ceil",
        signature: "ceil(n num) num",
        doc: "Ceiling.",
    },
    BuiltinDoc {
        name: "abs",
        signature: "abs(n num) num",
        doc: "Absolute value.",
    },
    BuiltinDoc {
        name: "round",
        signature: "round(n num) num",
        doc: "Round (halves toward +∞).",
    },
    BuiltinDoc {
        name: "tonum",
        signature: "tonum(s str) num",
        doc: "Parse string to number (tonumber).",
    },
    BuiltinDoc {
        name: "tostr",
        signature: "tostr(n num) str",
        doc: "Convert to string (tostring).",
    },
    BuiltinDoc {
        name: "push",
        signature: "push(list list, val)",
        doc: "Append value to list.",
    },
    BuiltinDoc {
        name: "pop",
        signature: "pop(list list)",
        doc: "Remove and return last element.",
    },
    BuiltinDoc {
        name: "has",
        signature: "has(list list, val) bool",
        doc: "Linear scan membership check.",
    },
    BuiltinDoc {
        name: "default",
        signature: "default(v, d)",
        doc: "Return d if v is nil or empty string.",
    },
    BuiltinDoc {
        name: "padleft",
        signature: "padleft(s str, n num, c str?) str",
        doc: "Left-pad string.",
    },
    BuiltinDoc {
        name: "padright",
        signature: "padright(s str, n num, c str?) str",
        doc: "Right-pad string.",
    },
    BuiltinDoc {
        name: "page",
        signature: "page(name str) title",
        doc: "Construct a MediaWiki title object (mw.title.new). Hoisted.",
    },
    BuiltinDoc {
        name: "currentpage",
        signature: "currentpage() title",
        doc: "Current page title (mw.title.getCurrentTitle).",
    },
    BuiltinDoc {
        name: "arg",
        signature: "arg(name, default?)",
        doc: "Read template argument. Trims, treats empty as nil.",
    },
    BuiltinDoc {
        name: "addWarning",
        signature: "addWarning(msg str)",
        doc: "mw.addWarning",
    },
    BuiltinDoc {
        name: "loadData",
        signature: "loadData(module str)",
        doc: "mw.loadData",
    },
    BuiltinDoc {
        name: "loadJsonData",
        signature: "loadJsonData(module str)",
        doc: "mw.loadJsonData",
    },
    BuiltinDoc {
        name: "allToString",
        signature: "allToString(...) str",
        doc: "mw.allToString",
    },
    BuiltinDoc {
        name: "clone",
        signature: "clone(val)",
        doc: "mw.clone",
    },
    BuiltinDoc {
        name: "dumpObject",
        signature: "dumpObject(val) str",
        doc: "mw.dumpObject",
    },
    BuiltinDoc {
        name: "log",
        signature: "log(val)",
        doc: "mw.log",
    },
    BuiltinDoc {
        name: "logObject",
        signature: "logObject(val)",
        doc: "mw.logObject",
    },
    BuiltinDoc {
        name: "getCurrentFrame",
        signature: "getCurrentFrame()",
        doc: "mw.getCurrentFrame()",
    },
    BuiltinDoc {
        name: "isSubsting",
        signature: "isSubsting() bool",
        doc: "mw.isSubsting",
    },
    BuiltinDoc {
        name: "incrementExpensiveFunctionCount",
        signature: "incrementExpensiveFunctionCount()",
        doc: "mw.incrementExpensiveFunctionCount",
    },
];

pub fn find_builtin(name: &str) -> Option<&'static BuiltinDoc> {
    BUILTINS.iter().find(|b| b.name == name)
}

pub fn keyword_doc(word: &str) -> Option<&'static str> {
    match word {
        "fnc" => Some("Function declaration"),
        "pub" => Some("Makes a function visible to other modules"),
        "import" => Some("Imports another Scribunto module"),
        "as" => Some("Type coercion or import alias"),
        "return" => Some("Return a value from the current function"),
        "output" => Some("Emit wikitext (string or raw block)"),
        "if" | "else" => Some("Conditional"),
        "for" => Some("Loop (for-in or numeric range)"),
        "in" => Some("Iteration clause"),
        "while" => Some("While loop"),
        "repeat" => Some("Repeat N times"),
        "break" | "continue" => Some("Loop control"),
        "try" | "catch" => Some("Error handling"),
        "lua" => Some("Raw Lua escape block"),
        "and" | "or" | "not" => Some("Logical operator"),
        "true" | "false" => Some("Boolean literal"),
        "nil" => Some("Nil / null value"),
        "str" | "num" | "bool" | "list" | "map" => Some("Type annotation"),
        _ => None,
    }
}
