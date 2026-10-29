//! GM8-style drag-and-drop action definitions and the Actions → GML compiler.
//! Action ids are provisional RGM ids; a 1:1 mapping table for .gmk import
//! comes with the importer (GM8 lib ids differ for some entries).

use gmformat::{DndAction, DndParam};

// ---------------------------------------------------------------- param kinds

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ParamKind {
    /// A number or GML expression (edited as text).
    Expr,
    /// Plain text.
    Text,
    /// GM color.
    Color,
    /// true/false checkbox.
    Bool,
    /// Dropdown with fixed options: (label, emitted GML string).
    Menu(&'static [(&'static str, &'static str)]),
    /// The 9-button direction grid of "Start moving in a direction" (bit mask).
    DirectionGrid,
    // resource selectors
    Sprite,
    Sound,
    Background,
    Path,
    Script,
    Timeline,
    Font,
    Object,
    Room,
}

pub struct ParamDef {
    pub name: &'static str,
    pub kind: ParamKind,
}

pub struct ActionDef {
    pub lib: u32,
    pub id: u32,
    pub name: &'static str,
    /// short description shown in tooltips
    pub hint: &'static str,
    /// question actions (render with "?", produce a condition)
    pub question: bool,
    /// supports the "relative" checkbox
    pub relative_ok: bool,
    pub params: &'static [ParamDef],
}

pub struct LibDef {
    pub id: u32,
    pub name: &'static str,
}

pub const LIBS: &[LibDef] = &[
    LibDef {
        id: 1,
        name: "move",
    },
    LibDef {
        id: 2,
        name: "main1",
    },
    LibDef {
        id: 3,
        name: "main2",
    },
    LibDef {
        id: 4,
        name: "control",
    },
    LibDef {
        id: 5,
        name: "score",
    },
    LibDef {
        id: 6,
        name: "extra",
    },
    LibDef {
        id: 7,
        name: "draw",
    },
    LibDef {
        id: 8,
        name: "particles",
    },
];

const E: ParamKind = ParamKind::Expr;
const T: ParamKind = ParamKind::Text;
const C: ParamKind = ParamKind::Color;
#[allow(dead_code)]
const B: ParamKind = ParamKind::Bool;

const CMP_MENU: &[(&str, &str)] = &[
    ("is equal to", "=="),
    ("is smaller than", "<"),
    ("is larger than", ">"),
    ("is smaller than or equal", "<="),
    ("is larger than or equal", ">="),
    ("is not equal to", "!="),
];

const BOOL_MENU: &[(&str, &str)] = &[("false", "0"), ("true", "1")];

pub const ACTIONS: &[ActionDef] = &[
    // ---------------- lib 1: move ----------------
    ActionDef {
        lib: 1,
        id: 121,
        name: "Start moving in a direction",
        hint: "directions grid + speed",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "directions",
                kind: ParamKind::DirectionGrid,
            },
            ParamDef {
                name: "speed",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 1,
        id: 122,
        name: "Set direction and speed of motion",
        hint: "motion_set / motion_add",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef {
                name: "direction",
                kind: E,
            },
            ParamDef {
                name: "speed",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 1,
        id: 123,
        name: "Move towards a point",
        hint: "move_towards_point",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "speed",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 1,
        id: 124,
        name: "Speed horizontal",
        hint: "hspeed",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "hspeed",
            kind: E,
        }],
    },
    ActionDef {
        lib: 1,
        id: 125,
        name: "Speed vertical",
        hint: "vspeed",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "vspeed",
            kind: E,
        }],
    },
    ActionDef {
        lib: 1,
        id: 126,
        name: "Set gravity",
        hint: "gravity direction + amount",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef {
                name: "direction",
                kind: E,
            },
            ParamDef {
                name: "amount",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 1,
        id: 127,
        name: "Reverse horizontal direction",
        hint: "hspeed = -hspeed",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 1,
        id: 128,
        name: "Reverse vertical direction",
        hint: "vspeed = -vspeed",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 1,
        id: 129,
        name: "Set friction",
        hint: "friction",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "friction",
            kind: E,
        }],
    },
    // ---------------- lib 2: main1 ----------------
    ActionDef {
        lib: 2,
        id: 131,
        name: "Create instance",
        hint: "instance_create",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 132,
        name: "Create moving",
        hint: "create + motion_set",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
            ParamDef {
                name: "speed",
                kind: E,
            },
            ParamDef {
                name: "direction",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 133,
        name: "Create random",
        hint: "random position in a rect",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "x1",
                kind: E,
            },
            ParamDef {
                name: "y1",
                kind: E,
            },
            ParamDef {
                name: "x2",
                kind: E,
            },
            ParamDef {
                name: "y2",
                kind: E,
            },
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 134,
        name: "Change instance",
        hint: "instance_change",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
            ParamDef {
                name: "perform events",
                kind: ParamKind::Menu(BOOL_MENU),
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 135,
        name: "Destroy instance",
        hint: "instance_destroy",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 2,
        id: 136,
        name: "Destroy at position",
        hint: "position_destroy",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
        ],
    },
    ActionDef {
        lib: 2,
        id: 137,
        name: "Change sprite",
        hint: "sprite_index etc.",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "sprite",
                kind: ParamKind::Sprite,
            },
            ParamDef {
                name: "subimage (-1 = keep)",
                kind: E,
            },
            ParamDef {
                name: "speed",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 138,
        name: "Transform sprite",
        hint: "scale/rotation",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "xscale",
                kind: E,
            },
            ParamDef {
                name: "yscale",
                kind: E,
            },
            ParamDef {
                name: "angle",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 139,
        name: "Set collision mask",
        hint: "mask_index",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "sprite (-1 = none)",
            kind: ParamKind::Sprite,
        }],
    },
    ActionDef {
        lib: 2,
        id: 141,
        name: "Play a sound",
        hint: "sound_play / sound_loop",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "sound",
                kind: ParamKind::Sound,
            },
            ParamDef {
                name: "loop",
                kind: ParamKind::Menu(BOOL_MENU),
            },
        ],
    },
    ActionDef {
        lib: 2,
        id: 142,
        name: "Stop a sound",
        hint: "sound_stop",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "sound",
            kind: ParamKind::Sound,
        }],
    },
    ActionDef {
        lib: 2,
        id: 143,
        name: "If a sound is playing",
        hint: "sound_isplaying",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "sound",
            kind: ParamKind::Sound,
        }],
    },
    ActionDef {
        lib: 2,
        id: 144,
        name: "Go to next room",
        hint: "room_goto_next",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 2,
        id: 145,
        name: "Go to previous room",
        hint: "room_goto_previous",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 2,
        id: 146,
        name: "Go to a different room",
        hint: "room_goto",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "room",
            kind: ParamKind::Room,
        }],
    },
    ActionDef {
        lib: 2,
        id: 147,
        name: "Restart the current room",
        hint: "room_restart",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 2,
        id: 148,
        name: "Sleep some milliseconds",
        hint: "sleep",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "milliseconds",
            kind: E,
        }],
    },
    ActionDef {
        lib: 2,
        id: 149,
        name: "Set the mouse cursor",
        hint: "cursor_sprite",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "sprite (-1 = default)",
            kind: ParamKind::Sprite,
        }],
    },
    // ---------------- lib 3: main2 ----------------
    ActionDef {
        lib: 3,
        id: 151,
        name: "Set the value of an alarm clock",
        hint: "alarm[n]",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef {
                name: "steps",
                kind: E,
            },
            ParamDef {
                name: "alarm nr",
                kind: ParamKind::Menu(&[
                    ("Alarm 0", "0"),
                    ("Alarm 1", "1"),
                    ("Alarm 2", "2"),
                    ("Alarm 3", "3"),
                    ("Alarm 4", "4"),
                    ("Alarm 5", "5"),
                    ("Alarm 6", "6"),
                    ("Alarm 7", "7"),
                    ("Alarm 8", "8"),
                    ("Alarm 9", "9"),
                    ("Alarm 10", "10"),
                    ("Alarm 11", "11"),
                ]),
            },
        ],
    },
    ActionDef {
        lib: 3,
        id: 152,
        name: "Use a path",
        hint: "path_start",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "path",
                kind: ParamKind::Path,
            },
            ParamDef {
                name: "speed",
                kind: E,
            },
            ParamDef {
                name: "at end",
                kind: ParamKind::Menu(&[("stop", "0"), ("restart", "1"), ("reverse", "2")]),
            },
            ParamDef {
                name: "absolute",
                kind: ParamKind::Menu(BOOL_MENU),
            },
        ],
    },
    ActionDef {
        lib: 3,
        id: 153,
        name: "Position in path",
        hint: "jump to nearest point",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "point (0..n-1)",
            kind: E,
        }],
    },
    ActionDef {
        lib: 3,
        id: 154,
        name: "Path position",
        hint: "path_position = fraction",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "position (0..1)",
            kind: E,
        }],
    },
    ActionDef {
        lib: 3,
        id: 155,
        name: "Path speed",
        hint: "path_speed",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "speed",
            kind: E,
        }],
    },
    ActionDef {
        lib: 3,
        id: 156,
        name: "Bounce against objects",
        hint: "move_bounce_solid",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "precisely",
            kind: ParamKind::Menu(BOOL_MENU),
        }],
    },
    // ---------------- lib 4: control ----------------
    ActionDef {
        lib: 4,
        id: 401,
        name: "Execute a piece of code",
        hint: "GML code",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "code",
            kind: T,
        }],
    },
    ActionDef {
        lib: 4,
        id: 402,
        name: "Comment",
        hint: "// text",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "comment",
            kind: T,
        }],
    },
    ActionDef {
        lib: 4,
        id: 403,
        name: "Set the value of a variable",
        hint: "var = value",
        question: false,
        relative_ok: true,
        params: &[
            ParamDef {
                name: "variable",
                kind: T,
            },
            ParamDef {
                name: "value",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 404,
        name: "If a variable has a value",
        hint: "compare a variable",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "variable",
                kind: T,
            },
            ParamDef {
                name: "comparison",
                kind: ParamKind::Menu(CMP_MENU),
            },
            ParamDef {
                name: "value",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 405,
        name: "If an expression is true",
        hint: "expression condition",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "expression",
            kind: E,
        }],
    },
    ActionDef {
        lib: 4,
        id: 406,
        name: "Check empty",
        hint: "place is free at position",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "objects",
                kind: ParamKind::Menu(&[("only solid", "1"), ("all", "0")]),
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 407,
        name: "Check collision",
        hint: "not empty at position",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "objects",
                kind: ParamKind::Menu(&[("only solid", "1"), ("all", "0")]),
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 408,
        name: "Check object",
        hint: "object at position",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 409,
        name: "Test instance count",
        hint: "number of instances",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "object",
                kind: ParamKind::Object,
            },
            ParamDef {
                name: "comparison",
                kind: ParamKind::Menu(CMP_MENU),
            },
            ParamDef {
                name: "number",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 4,
        id: 410,
        name: "Test chance",
        hint: "1 in n chance",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "1 out of",
            kind: E,
        }],
    },
    ActionDef {
        lib: 4,
        id: 411,
        name: "Ask a question",
        hint: "yes/no dialog",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "question",
            kind: T,
        }],
    },
    ActionDef {
        lib: 4,
        id: 412,
        name: "Test expression",
        hint: "expression value is true",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "expression",
            kind: E,
        }],
    },
    ActionDef {
        lib: 4,
        id: 413,
        name: "Check mouse",
        hint: "mouse button down",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "button",
            kind: ParamKind::Menu(&[
                ("left", "1"),
                ("right", "2"),
                ("middle", "4"),
                ("none", "0"),
                ("any", "-1"),
            ]),
        }],
    },
    ActionDef {
        lib: 4,
        id: 414,
        name: "Check grid",
        hint: "aligned to grid",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "hsnap",
                kind: E,
            },
            ParamDef {
                name: "vsnap",
                kind: E,
            },
        ],
    },
    // markers (kind field carries the real semantics; these defs exist for the palette)
    ActionDef {
        lib: 4,
        id: 420,
        name: "Start of a block",
        hint: "{",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 4,
        id: 421,
        name: "End of a block",
        hint: "}",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 4,
        id: 422,
        name: "Else",
        hint: "} else {",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 4,
        id: 423,
        name: "Exit this event",
        hint: "exit",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 4,
        id: 424,
        name: "Repeat next action",
        hint: "repeat (n)",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "times",
            kind: E,
        }],
    },
    ActionDef {
        lib: 4,
        id: 425,
        name: "While next action",
        hint: "while (condition)",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "condition",
            kind: E,
        }],
    },
    ActionDef {
        lib: 4,
        id: 426,
        name: "Until next action",
        hint: "do .. until",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "condition",
            kind: E,
        }],
    },
    // ---------------- lib 5: score ----------------
    ActionDef {
        lib: 5,
        id: 501,
        name: "Set the score",
        hint: "score",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "new score",
            kind: E,
        }],
    },
    ActionDef {
        lib: 5,
        id: 502,
        name: "If score has a value",
        hint: "compare score",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "comparison",
                kind: ParamKind::Menu(CMP_MENU),
            },
            ParamDef {
                name: "value",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 5,
        id: 503,
        name: "Draw the score",
        hint: "caption + score at position",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "caption",
                kind: T,
            },
        ],
    },
    ActionDef {
        lib: 5,
        id: 504,
        name: "Set the number of lives",
        hint: "lives",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "new lives",
            kind: E,
        }],
    },
    ActionDef {
        lib: 5,
        id: 505,
        name: "If lives have a value",
        hint: "compare lives",
        question: true,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "comparison",
                kind: ParamKind::Menu(CMP_MENU),
            },
            ParamDef {
                name: "value",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 5,
        id: 506,
        name: "Draw the number of lives",
        hint: "caption + lives",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "caption",
                kind: T,
            },
        ],
    },
    ActionDef {
        lib: 5,
        id: 507,
        name: "Set the health",
        hint: "health",
        question: false,
        relative_ok: true,
        params: &[ParamDef {
            name: "new health",
            kind: E,
        }],
    },
    ActionDef {
        lib: 5,
        id: 508,
        name: "If health is less than",
        hint: "health <",
        question: true,
        relative_ok: false,
        params: &[ParamDef {
            name: "value",
            kind: E,
        }],
    },
    ActionDef {
        lib: 5,
        id: 509,
        name: "Draw the health bar",
        hint: "bar at position",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "x1",
                kind: E,
            },
            ParamDef {
                name: "y1",
                kind: E,
            },
            ParamDef {
                name: "x2",
                kind: E,
            },
            ParamDef {
                name: "y2",
                kind: E,
            },
            ParamDef {
                name: "back color",
                kind: C,
            },
            ParamDef {
                name: "min color",
                kind: C,
            },
            ParamDef {
                name: "max color",
                kind: C,
            },
        ],
    },
    ActionDef {
        lib: 5,
        id: 510,
        name: "Clear the highscore table",
        hint: "highscore_clear",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 5,
        id: 511,
        name: "If the score is among the best",
        hint: "top 10 check",
        question: true,
        relative_ok: false,
        params: &[],
    },
    // ---------------- lib 6: extra (stubs where noted) ----------------
    ActionDef {
        lib: 6,
        id: 601,
        name: "Open a web page",
        hint: "url_open",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "url",
            kind: T,
        }],
    },
    ActionDef {
        lib: 6,
        id: 602,
        name: "Show the game information",
        hint: "info dialog (stub)",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 6,
        id: 603,
        name: "Show a video",
        hint: "video (stub)",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "file",
            kind: T,
        }],
    },
    ActionDef {
        lib: 6,
        id: 611,
        name: "Play a CD",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 6,
        id: 612,
        name: "Pause the CD",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 6,
        id: 613,
        name: "Resume the CD",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 6,
        id: 614,
        name: "Stop the CD",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 6,
        id: 621,
        name: "Replace sprite from file",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "sprite",
                kind: ParamKind::Sprite,
            },
            ParamDef {
                name: "file",
                kind: T,
            },
        ],
    },
    ActionDef {
        lib: 6,
        id: 622,
        name: "Replace background from file",
        hint: "stub",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "background",
                kind: ParamKind::Background,
            },
            ParamDef {
                name: "file",
                kind: T,
            },
        ],
    },
    // ---------------- lib 7: draw ----------------
    ActionDef {
        lib: 7,
        id: 701,
        name: "Draw a sprite image",
        hint: "draw_sprite",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "sprite",
                kind: ParamKind::Sprite,
            },
            ParamDef {
                name: "subimage (-1 = current)",
                kind: E,
            },
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
        ],
    },
    ActionDef {
        lib: 7,
        id: 702,
        name: "Draw a background image",
        hint: "draw_background",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "background",
                kind: ParamKind::Background,
            },
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
        ],
    },
    ActionDef {
        lib: 7,
        id: 703,
        name: "Draw a text",
        hint: "draw_text",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "text",
                kind: T,
            },
        ],
    },
    ActionDef {
        lib: 7,
        id: 704,
        name: "Draw a transformed text",
        hint: "draw_text_transformed",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "text",
                kind: T,
            },
            ParamDef {
                name: "xscale",
                kind: E,
            },
            ParamDef {
                name: "yscale",
                kind: E,
            },
            ParamDef {
                name: "angle",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 7,
        id: 705,
        name: "Draw a rectangle",
        hint: "draw_rectangle",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "x1",
                kind: E,
            },
            ParamDef {
                name: "y1",
                kind: E,
            },
            ParamDef {
                name: "x2",
                kind: E,
            },
            ParamDef {
                name: "y2",
                kind: E,
            },
            ParamDef {
                name: "filled",
                kind: ParamKind::Menu(BOOL_MENU),
            },
        ],
    },
    ActionDef {
        lib: 7,
        id: 706,
        name: "Draw a horizontal gradient",
        hint: "draw_rectangle_color",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "x1",
                kind: E,
            },
            ParamDef {
                name: "y1",
                kind: E,
            },
            ParamDef {
                name: "x2",
                kind: E,
            },
            ParamDef {
                name: "y2",
                kind: E,
            },
            ParamDef {
                name: "color 1",
                kind: C,
            },
            ParamDef {
                name: "color 2",
                kind: C,
            },
        ],
    },
    ActionDef {
        lib: 7,
        id: 707,
        name: "Draw a vertical gradient",
        hint: "draw_rectangle_color",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "x1",
                kind: E,
            },
            ParamDef {
                name: "y1",
                kind: E,
            },
            ParamDef {
                name: "x2",
                kind: E,
            },
            ParamDef {
                name: "y2",
                kind: E,
            },
            ParamDef {
                name: "color 1",
                kind: C,
            },
            ParamDef {
                name: "color 2",
                kind: C,
            },
        ],
    },
    ActionDef {
        lib: 7,
        id: 708,
        name: "Set the color",
        hint: "draw_set_color",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "color",
            kind: C,
        }],
    },
    ActionDef {
        lib: 7,
        id: 709,
        name: "Set the font",
        hint: "draw_set_font",
        question: false,
        relative_ok: false,
        params: &[ParamDef {
            name: "font (-1 = default)",
            kind: ParamKind::Font,
        }],
    },
    ActionDef {
        lib: 7,
        id: 710,
        name: "Draw the value of a variable",
        hint: "caption + value",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "caption",
                kind: T,
            },
            ParamDef {
                name: "variable",
                kind: T,
            },
        ],
    },
    // ---------------- lib 8: particles (minimal) ----------------
    ActionDef {
        lib: 8,
        id: 801,
        name: "Create particle system",
        hint: "global.rgm_ps",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 8,
        id: 802,
        name: "Destroy particle system",
        hint: "global.rgm_ps",
        question: false,
        relative_ok: false,
        params: &[],
    },
    ActionDef {
        lib: 8,
        id: 811,
        name: "Create particle type",
        hint: "stored in a variable",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "variable",
                kind: T,
            },
            ParamDef {
                name: "shape",
                kind: ParamKind::Menu(&[
                    ("pixel", "0"),
                    ("disk", "1"),
                    ("square", "2"),
                    ("line", "3"),
                    ("star", "4"),
                    ("circle", "5"),
                    ("ring", "6"),
                    ("sphere", "7"),
                    ("flare", "8"),
                    ("spark", "9"),
                ]),
            },
            ParamDef {
                name: "size min",
                kind: E,
            },
            ParamDef {
                name: "size max",
                kind: E,
            },
            ParamDef {
                name: "color",
                kind: C,
            },
            ParamDef {
                name: "life min",
                kind: E,
            },
            ParamDef {
                name: "life max",
                kind: E,
            },
        ],
    },
    ActionDef {
        lib: 8,
        id: 812,
        name: "Create particles at position",
        hint: "burst",
        question: false,
        relative_ok: false,
        params: &[
            ParamDef {
                name: "type variable",
                kind: T,
            },
            ParamDef { name: "x", kind: E },
            ParamDef { name: "y", kind: E },
            ParamDef {
                name: "count",
                kind: E,
            },
        ],
    },
];

pub fn find_def(lib: u32, id: u32) -> Option<&'static ActionDef> {
    ACTIONS.iter().find(|a| a.lib == lib && a.id == id)
}

// ---------------------------------------------------------------- compiler

/// Compile a D&D action list into GML source.
/// `objects` resolves `applies_to` object indices into names.
pub fn compile_event(
    actions: &[DndAction],
    objects: &[gmformat::GmObject],
) -> Result<String, String> {
    let mut i = 0usize;
    let stmts = parse_level(actions, &mut i, objects, true)?;
    if i < actions.len() {
        return Err(format!(
            "'End of block' without a matching 'Start' (action #{})",
            i + 1
        ));
    }
    let mut out = String::new();
    for (text, _) in stmts {
        out.push_str(&text);
    }
    Ok(out)
}

/// One statement of a block level: its GML text + whether it is an
/// if-construct (can take `else`).
type LevelStmt = (String, bool);

fn parse_level(
    actions: &[DndAction],
    i: &mut usize,
    objects: &[gmformat::GmObject],
    top: bool,
) -> Result<Vec<LevelStmt>, String> {
    let mut out: Vec<LevelStmt> = Vec::new();
    // pending wrappers waiting for the next statement/block
    let mut pending_q: Option<((String, bool), &DndAction)> = None; // condition + question action
    let mut pending_rep: Option<(String, &DndAction)> = None;
    let mut pending_loop: Option<(String, bool, &DndAction)> = None; // (cond, is_while, action)

    while *i < actions.len() {
        let a = &actions[*i];
        let def = find_def(a.lib, a.id);
        match a.kind {
            1 => {
                // Begin block: parse the inner level, then wrap with pendings
                *i += 1;
                let inner = parse_level(actions, i, objects, false)?;
                let mut inner_text = String::new();
                for (t, _) in inner {
                    inner_text.push_str(&t);
                }
                let mut wrapped = format!("{{\n{}}}", inner_text);
                let head_q = pending_q.take();
                let head_rep = pending_rep.take();
                let head_loop = pending_loop.take();
                let is_if = head_q.is_some();
                let owner = head_q
                    .as_ref()
                    .map(|(_, a)| a)
                    .or(head_rep.as_ref().map(|(_, a)| a))
                    .or(head_loop.as_ref().map(|(_, _, a)| a));
                if let Some(((c, neg), _)) = &head_q {
                    let c = c.clone();
                    let neg = *neg;
                    wrapped = format!("if {} {}", wrap_cond(&c, neg), wrapped);
                } else if let Some((n, _)) = &head_rep {
                    let n = n.clone();
                    wrapped = format!("repeat ({}) {}", n, wrapped);
                } else if let Some((c, is_while, _)) = &head_loop {
                    let c = c.clone();
                    let is_while = *is_while;
                    if is_while {
                        wrapped = format!("while ({}) {}", c, wrapped);
                    } else {
                        // move the closing brace into the until-clause
                        let inner_len = wrapped.len();
                        let body = wrapped[..inner_len - 1].to_string();
                        wrapped = format!("do {} until (!({}));", body, c);
                    }
                }
                let final_text = match owner {
                    Some(owner) => apply_with(owner, wrapped, objects)?,
                    None => wrapped,
                };
                out.push((final_text + "\n", is_if));
            }
            2 => {
                // End of block: return to the caller (Begin handler)
                if top {
                    return Err("'End of block' without a matching 'Start'".into());
                }
                if pending_q.is_some() || pending_rep.is_some() || pending_loop.is_some() {
                    return Err("a question/repeat has no following action or block".into());
                }
                *i += 1;
                return Ok(out);
            }
            3 => {
                // Else: attach to the previous if-construct
                let Some((if_text, true)) = out.pop() else {
                    return Err("'Else' must follow a question action or its block".into());
                };
                *i += 1;
                // parse the else body (a following block or a single action)
                let (else_text, _) = parse_single(actions, i, objects)?;
                let merged = merge_else(&if_text, &else_text)?;
                out.push((merged, true));
            }
            4 => {
                let stmt = "exit;".to_string();
                push_wrapped_statement(
                    &mut out,
                    a,
                    stmt,
                    &mut pending_q,
                    &mut pending_rep,
                    &mut pending_loop,
                    objects,
                )?;
                *i += 1;
            }
            5 => {
                pending_rep = Some((num_or_expr(a, 0, "1"), a));
                *i += 1;
            }
            6 => {
                pending_loop = Some((pexpr(a, 0, "1"), true, a));
                *i += 1;
            }
            7 => {
                pending_loop = Some((pexpr(a, 0, "1"), false, a));
                *i += 1;
            }
            _ => {
                let Some(def) = def else {
                    return Err(format!("unknown action lib {} id {}", a.lib, a.id));
                };
                if def.question {
                    if pending_q.is_some() {
                        return Err(format!(
                            "question '{}' directly follows another question",
                            def.name
                        ));
                    }
                    let cond = emit_condition(a, def)?;
                    pending_q = Some(((cond, a.not), a));
                    *i += 1;
                    continue;
                }
                let stmt = emit_action(a, def)?;
                push_wrapped_statement(
                    &mut out,
                    a,
                    stmt,
                    &mut pending_q,
                    &mut pending_rep,
                    &mut pending_loop,
                    objects,
                )?;
                *i += 1;
            }
        }
    }
    if pending_q.is_some() || pending_rep.is_some() || pending_loop.is_some() {
        return Err("a question/repeat at the end of the event has no body".into());
    }
    Ok(out)
}

/// Parse the next single construct (a Begin..End block or one action),
/// used for the else-branch body.
fn parse_single(
    actions: &[DndAction],
    i: &mut usize,
    objects: &[gmformat::GmObject],
) -> Result<LevelStmt, String> {
    if *i >= actions.len() {
        return Ok(("/* empty */\n".into(), false));
    }
    let a = &actions[*i];
    if a.kind == 1 {
        let mut level = parse_level(actions, i, objects, false)?;
        if level.len() == 1 {
            return Ok(level.remove(0));
        }
        let mut text = String::from("{\n");
        for (t, _) in level {
            text.push_str(&t);
        }
        return Ok((text + "}\n", false));
    }
    // a single action (ignore its own pendings — not valid there)
    let mut dummy_q = None;
    let mut dummy_rep = None;
    let mut dummy_loop = None;
    let mut sink: Vec<LevelStmt> = Vec::new();
    let j = *i;
    // temporarily treat as normal/question statement
    let a = &actions[j];
    if a.kind == 3 || a.kind == 2 {
        return Err("'Else' branch body must be an action or a block".into());
    }
    let stmt = if a.kind == 4 {
        "exit;\n".to_string()
    } else {
        let def = find_def(a.lib, a.id)
            .ok_or_else(|| format!("unknown action lib {} id {}", a.lib, a.id))?;
        if def.question {
            // nested question in else body: parse a question + its body pair
            let cond = emit_condition(a, def)?;
            *i += 1;
            let (body, _) = parse_single(actions, i, objects)?;
            return Ok((
                format!("if {} {}", wrap_cond(&cond, a.not), body.trim_end()),
                false,
            ));
        }
        emit_action(a, def)? + "\n"
    };
    push_wrapped_statement(
        &mut sink,
        a,
        stmt,
        &mut dummy_q,
        &mut dummy_rep,
        &mut dummy_loop,
        objects,
    )?;
    *i += 1;
    Ok(sink.remove(0))
}

/// Emit one statement, wrapping it with any pending question/repeat/loop.
#[allow(clippy::too_many_arguments)]
fn push_wrapped_statement(
    out: &mut Vec<LevelStmt>,
    a: &DndAction,
    stmt: String,
    pending_q: &mut Option<((String, bool), &DndAction)>,
    pending_rep: &mut Option<(String, &DndAction)>,
    pending_loop: &mut Option<(String, bool, &DndAction)>,
    objects: &[gmformat::GmObject],
) -> Result<(), String> {
    let mut wrapped = stmt;
    let mut is_if = false;
    let mut owner: Option<&DndAction> = None;
    if let Some(((c, neg), qa)) = pending_q.take() {
        wrapped = format!("if {} {}", wrap_cond(&c, neg), wrapped.trim_end());
        is_if = true;
        owner = Some(qa);
    } else if let Some((n, ra)) = pending_rep.take() {
        wrapped = format!("repeat ({}) {{\n{}}}", n, wrapped);
        owner = Some(ra);
    } else if let Some((c, is_while, la)) = pending_loop.take() {
        if is_while {
            wrapped = format!("while ({}) {{\n{}}}", c, wrapped);
        } else {
            wrapped = format!("do {{\n{}}} until (!({}));", wrapped, c);
        }
        owner = Some(la);
    }
    let final_text = match owner {
        Some(owner) => apply_with(owner, wrapped, objects)?,
        None => apply_with(a, wrapped, objects)?,
    };
    out.push((final_text + "\n", is_if));
    Ok(())
}

/// Attach an else-branch to an emitted if-construct.
fn merge_else(if_text: &str, else_text: &str) -> Result<String, String> {
    let trimmed = if_text.trim_end();
    if trimmed.ends_with('}') {
        // block-if: "…}" → "…} else { …}"
        Ok(format!("{} else {{\n{}}}\n", trimmed, else_text))
    } else if let Some(pos) = trimmed.find("if ") {
        // single-statement if: "if (c) stmt" → "if (c) stmt else stmt"
        let _ = pos;
        Ok(format!("{} else {}\n", trimmed, else_text.trim_end()))
    } else {
        Err("'Else' must follow a question".into())
    }
}

/// Wrap emitted GML in a `with` when applies_to says so.
fn apply_with(
    a: &DndAction,
    stmt: String,
    objects: &[gmformat::GmObject],
) -> Result<String, String> {
    let target = match a.applies_to {
        -1 => return Ok(stmt),
        -2 => "other".to_string(),
        idx => {
            let idx = idx as usize;
            objects
                .get(idx)
                .map(|o| o.name.clone())
                .ok_or_else(|| format!("'applies to' object index {} does not exist", idx))?
        }
    };
    Ok(format!("with ({}) {{\n{}}}", target, stmt))
}

fn wrap_cond(c: &str, neg: bool) -> String {
    if neg {
        format!("!({})", c)
    } else {
        format!("({})", c)
    }
}

// ---------------------------------------------------------------- params

/// Expression param: string part when present, else the number.
fn pexpr(a: &DndAction, i: usize, default: &str) -> String {
    match a.params.get(i) {
        Some(DndParam::Real(x)) => fmt_num(*x),
        Some(DndParam::Str(s)) => {
            if s.is_empty() {
                default.to_string()
            } else {
                s.clone()
            }
        }
        Some(DndParam::Both { real, str: s }) => {
            if !s.is_empty() {
                s.clone()
            } else {
                fmt_num(*real)
            }
        }
        None => default.to_string(),
    }
}

fn num_or_expr(a: &DndAction, i: usize, default: &str) -> String {
    pexpr(a, i, default)
}

fn pnum(a: &DndAction, i: usize, default: f64) -> f64 {
    match a.params.get(i) {
        Some(DndParam::Real(x)) => *x,
        Some(DndParam::Both { real, .. }) => *real,
        Some(DndParam::Str(s)) => s.parse().unwrap_or(default),
        None => default,
    }
}

fn pstr(a: &DndAction, i: usize, default: &str) -> String {
    match a.params.get(i) {
        Some(DndParam::Str(s)) => s.clone(),
        Some(DndParam::Both { str: s, .. }) => s.clone(),
        Some(DndParam::Real(x)) => fmt_num(*x),
        None => default.to_string(),
    }
}

fn pmenu(a: &DndAction, i: usize, default: &str) -> String {
    pexpr(a, i, default)
}

fn fmt_num(x: f64) -> String {
    if x.fract().abs() < 1e-9 {
        format!("{}", x as i64)
    } else {
        format!("{}", x)
    }
}

fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn set_or_add(var: &str, value: &str, relative: bool) -> String {
    if relative {
        format!("{} += {};\n", var, value)
    } else {
        format!("{} = {};\n", var, value)
    }
}

fn maybe_rel_xy(a: &DndAction, xi: usize, yi: usize) -> (String, String) {
    if a.relative {
        (
            format!("x + {}", pexpr(a, xi, "0")),
            format!("y + {}", pexpr(a, yi, "0")),
        )
    } else {
        (pexpr(a, xi, "0"), pexpr(a, yi, "0"))
    }
}

fn sanitize_var(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ---------------------------------------------------------------- emitters

fn emit_condition(a: &DndAction, _def: &ActionDef) -> Result<String, String> {
    let c = match (a.lib, a.id) {
        (2, 143) => format!("sound_isplaying({})", pexpr(a, 0, "0")),
        (4, 404) => format!(
            "{} {} {}",
            pstr(a, 0, "x"),
            pmenu(a, 1, "=="),
            pexpr(a, 2, "0")
        ),
        (4, 405) | (4, 412) => pexpr(a, 0, "1"),
        (4, 406) => {
            let solid_only = pmenu(a, 2, "1") == "1";
            if solid_only {
                format!("place_free({}, {})", pexpr(a, 0, "0"), pexpr(a, 1, "0"))
            } else {
                format!("place_empty({}, {})", pexpr(a, 0, "0"), pexpr(a, 1, "0"))
            }
        }
        (4, 407) => {
            let solid_only = pmenu(a, 2, "1") == "1";
            if solid_only {
                format!("!place_free({}, {})", pexpr(a, 0, "0"), pexpr(a, 1, "0"))
            } else {
                format!("!place_empty({}, {})", pexpr(a, 0, "0"), pexpr(a, 1, "0"))
            }
        }
        (4, 408) => format!(
            "place_meeting({}, {}, {})",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "0")
        ),
        (4, 409) => format!(
            "instance_number({}) {} {}",
            pexpr(a, 0, "-1"),
            pmenu(a, 1, "=="),
            pexpr(a, 2, "0")
        ),
        (4, 410) => format!(
            "random({}) < 1",
            pexpr(a, 0, "2").parse::<f64>().unwrap_or(2.0).max(1.0) as i64
        ),
        (4, 411) => format!("show_question({})", q(&pstr(a, 0, "Continue?"))),
        (4, 413) => {
            let b = pmenu(a, 0, "1");
            if b == "0" {
                "mouse_button == 0".into()
            } else if b == "-1" {
                "mouse_button != 0".into()
            } else {
                format!("mouse_button == {}", b)
            }
        }
        (4, 414) => format!(
            "place_snapped({}, {})",
            pexpr(a, 0, "16"),
            pexpr(a, 1, "16")
        ),
        (5, 502) => format!("score {} {}", pmenu(a, 0, "=="), pexpr(a, 1, "0")),
        (5, 505) => format!("lives {} {}", pmenu(a, 0, "=="), pexpr(a, 1, "0")),
        (5, 508) => format!("health < {}", pexpr(a, 0, "0")),
        (5, 511) => "highscore_position(score) <= 10".into(),
        _ => return Err(format!("unknown question action lib {} id {}", a.lib, a.id)),
    };
    Ok(c)
}

fn emit_action(a: &DndAction, def: &ActionDef) -> Result<String, String> {
    let s = match (a.lib, a.id) {
        // -------- move --------
        (1, 121) => {
            let mask = pnum(a, 0, 32.0) as i32;
            let speed = pexpr(a, 1, "0");
            // GM8 grid layout: bit 0 = up-left … bit 4 = stop … bit 8 = down-right
            const GRID: [i32; 9] = [135, 90, 45, 180, -1, 0, 225, 270, 315];
            let picks: Vec<String> = GRID
                .iter()
                .enumerate()
                .filter(|(i, d)| **d >= 0 && (mask & (1 << i)) != 0)
                .map(|(_, d)| d.to_string())
                .collect();
            if mask == 16 || picks.is_empty() {
                // center = stop
                "speed = 0;\n".to_string()
            } else if picks.len() == 1 {
                format!("motion_set({}, {});\n", picks[0], speed)
            } else {
                format!("motion_set(choose({}), {});\n", picks.join(", "), speed)
            }
        }
        (1, 122) => {
            let d = pexpr(a, 0, "0");
            let sp = pexpr(a, 1, "0");
            if a.relative {
                format!("motion_add({}, {});\n", d, sp)
            } else {
                format!("motion_set({}, {});\n", d, sp)
            }
        }
        (1, 123) => format!(
            "move_towards_point({}, {}, {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "0")
        ),
        (1, 124) => set_or_add("hspeed", &pexpr(a, 0, "0"), a.relative),
        (1, 125) => set_or_add("vspeed", &pexpr(a, 0, "0"), a.relative),
        (1, 126) => {
            let amount = if a.relative {
                format!("gravity + {}", pexpr(a, 1, "0"))
            } else {
                pexpr(a, 1, "0")
            };
            format!("gravity_direction = {};\ngravity = {};\n", pexpr(a, 0, "270"), amount)
        }
        (1, 127) => "hspeed = -hspeed;\n".into(),
        (1, 128) => "vspeed = -vspeed;\n".into(),
        (1, 129) => set_or_add("friction", &pexpr(a, 0, "0"), a.relative),
        // -------- main1 --------
        (2, 131) => {
            let (x, y) = maybe_rel_xy(a, 0, 1);
            format!("instance_create({}, {}, {});\n", x, y, pexpr(a, 2, "0"))
        }
        (2, 132) => {
            let (x, y) = maybe_rel_xy(a, 0, 1);
            format!(
                "{{\n    var __i;\n    __i = instance_create({}, {}, {});\n    with (__i) motion_set({}, {});\n}}\n",
                x,
                y,
                pexpr(a, 2, "0"),
                pexpr(a, 4, "0"),
                pexpr(a, 3, "1")
            )
        }
        (2, 133) => format!(
            "instance_create({} + random({} - {}), {} + random({} - {}), {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 2, "0"),
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 3, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 4, "0")
        ),
        (2, 134) => format!("instance_change({}, {});\n", pexpr(a, 0, "0"), pmenu(a, 1, "1")),
        (2, 135) => "instance_destroy();\n".into(),
        (2, 136) => format!("position_destroy({}, {});\n", pexpr(a, 0, "0"), pexpr(a, 1, "0")),
        (2, 137) => {
            let sub = pexpr(a, 1, "-1");
            let sub_stmt = if sub == "-1" { String::new() } else { format!("image_index = {};\n", sub) };
            format!(
                "sprite_index = {};\n{}image_speed = {};\n",
                pexpr(a, 0, "-1"),
                sub_stmt,
                pexpr(a, 2, "1")
            )
        }
        (2, 138) => format!(
            "image_xscale = {};\nimage_yscale = {};\nimage_angle = {};\n",
            pexpr(a, 0, "1"),
            pexpr(a, 1, "1"),
            pexpr(a, 2, "0")
        ),
        (2, 139) => format!("mask_index = {};\n", pexpr(a, 0, "-1")),
        (2, 141) => {
            if pmenu(a, 1, "0") == "1" {
                format!("sound_loop({});\n", pexpr(a, 0, "0"))
            } else {
                format!("sound_play({});\n", pexpr(a, 0, "0"))
            }
        }
        (2, 142) => format!("sound_stop({});\n", pexpr(a, 0, "0")),
        (2, 144) => "room_goto_next();\n".into(),
        (2, 145) => "room_goto_previous();\n".into(),
        (2, 146) => format!("room_goto({});\n", pexpr(a, 0, "0")),
        (2, 147) => "room_restart();\n".into(),
        (2, 148) => format!("sleep({});\n", pexpr(a, 0, "0")),
        (2, 149) => format!("cursor_sprite = {};\n", pexpr(a, 0, "-1")),
        // -------- main2 --------
        (3, 151) => set_or_add(&format!("alarm[{}]", pmenu(a, 1, "0")), &pexpr(a, 0, "0"), a.relative),
        (3, 152) => format!(
            "path_start({}, {}, {}, {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "1"),
            pexpr(a, 2, "0"),
            pmenu(a, 3, "1")
        ),
        (3, 153) => format!(
            "path_position = {} / path_get_number(path_index);\n",
            pexpr(a, 0, "0")
        ),
        (3, 154) => format!("path_position = {};\n", pexpr(a, 0, "0")),
        (3, 155) => set_or_add("path_speed", &pexpr(a, 0, "0"), a.relative),
        (3, 156) => format!("move_bounce_solid({});\n", pmenu(a, 0, "0")),
        // -------- control --------
        (4, 401) => format!("{}\n", pstr(a, 0, "")),
        (4, 402) => {
            let txt = pstr(a, 0, "");
            let mut s = String::new();
            for line in txt.lines() {
                s.push_str(&format!("// {}\n", line));
            }
            if s.is_empty() {
                "//\n".into()
            } else {
                s
            }
        }
        (4, 403) => set_or_add(&pstr(a, 0, "x"), &pexpr(a, 1, "0"), a.relative),
        (4, 423) => "exit;\n".into(),
        // -------- score --------
        (5, 501) => set_or_add("score", &pexpr(a, 0, "0"), a.relative),
        (5, 503) => format!(
            "draw_text({}, {}, {} + string(score));\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            q(&pstr(a, 2, "Score: "))
        ),
        (5, 504) => set_or_add("lives", &pexpr(a, 0, "0"), a.relative),
        (5, 506) => format!(
            "draw_text({}, {}, {} + string(lives));\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            q(&pstr(a, 2, "Lives: "))
        ),
        (5, 507) => set_or_add("health", &pexpr(a, 0, "100"), a.relative),
        (5, 509) => format!(
            "draw_healthbar({}, {}, {}, {}, health, {}, {}, {}, 0, 1);\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "100"),
            pexpr(a, 3, "10"),
            pexpr(a, 4, "8421504"),
            pexpr(a, 5, "255"),
            pexpr(a, 6, "65280")
        ),
        (5, 510) => "highscore_clear();\n".into(),
        // -------- extra --------
        (6, 601) => format!("url_open({});\n", q(&pstr(a, 0, "https://example.com"))),
        (6, 602) => "show_debug_message(\"game information\");\n".into(),
        (6, 603) => format!("show_debug_message(\"video: {}\");\n", pstr(a, 0, "")),
        (6, 611) | (6, 612) | (6, 613) | (6, 614) => "/* cd action (not supported) */\n".into(),
        (6, 621) => "/* replace sprite from file (not supported) */\n".into(),
        (6, 622) => "/* replace background from file (not supported) */\n".into(),
        // -------- draw --------
        (7, 701) => format!(
            "draw_sprite({}, {}, {}, {});\n",
            pexpr(a, 0, "-1"),
            pexpr(a, 1, "-1"),
            pexpr(a, 2, "0"),
            pexpr(a, 3, "0")
        ),
        (7, 702) => format!(
            "draw_background({}, {}, {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "0")
        ),
        (7, 703) => format!("draw_text({}, {}, {});\n", pexpr(a, 0, "0"), pexpr(a, 1, "0"), q(&pstr(a, 2, ""))),
        (7, 704) => format!(
            "draw_text_transformed({}, {}, {}, {}, {}, {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            q(&pstr(a, 2, "")),
            pexpr(a, 3, "1"),
            pexpr(a, 4, "1"),
            pexpr(a, 5, "0")
        ),
        (7, 705) => format!(
            "draw_rectangle({}, {}, {}, {}, {});\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "100"),
            pexpr(a, 3, "100"),
            pmenu(a, 4, "0")
        ),
        (7, 706) => format!(
            "draw_rectangle_color({}, {}, {}, {}, {}, {}, 1);\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "100"),
            pexpr(a, 3, "100"),
            pexpr(a, 4, "0"),
            pexpr(a, 5, "16777215")
        ),
        (7, 707) => format!(
            "draw_rectangle_color({}, {}, {}, {}, {}, {}, 1);\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            pexpr(a, 2, "100"),
            pexpr(a, 3, "100"),
            pexpr(a, 4, "0"),
            pexpr(a, 5, "16777215")
        ),
        (7, 708) => format!("draw_set_color({});\n", pexpr(a, 0, "16777215")),
        (7, 709) => format!("draw_set_font({});\n", pexpr(a, 0, "-1")),
        (7, 710) => format!(
            "draw_text({}, {}, {} + string({}));\n",
            pexpr(a, 0, "0"),
            pexpr(a, 1, "0"),
            q(&pstr(a, 2, "")),
            pstr(a, 3, "x")
        ),
        // -------- particles --------
        (8, 801) => "if (!variable_global_exists(\"rgm_ps\")) global.rgm_ps = part_system_create();\n".into(),
        (8, 802) => "if (variable_global_exists(\"rgm_ps\")) part_system_destroy(global.rgm_ps);\n".into(),
        (8, 811) => format!(
            "{{\n    var __t;\n    __t = part_type_create(global.rgm_ps);\n    part_type_shape(__t, {});\n    part_type_size(__t, {}, {}, 0, 0);\n    part_type_color1(__t, {});\n    part_type_life(__t, {}, {});\n    global.{} = __t;\n}}\n",
            pmenu(a, 1, "0"),
            pexpr(a, 2, "1"),
            pexpr(a, 3, "2"),
            pexpr(a, 4, "16777215"),
            pexpr(a, 5, "30"),
            pexpr(a, 6, "60"),
            sanitize_var(&pstr(a, 0, "ptype"))
        ),
        (8, 812) => format!(
            "part_particles_create(global.rgm_ps, {}, {}, global.{}, {});\n",
            pexpr(a, 1, "0"),
            pexpr(a, 2, "0"),
            sanitize_var(&pstr(a, 0, "ptype")),
            pexpr(a, 3, "10")
        ),
        _ => return Err(format!("action '{}' (lib {} id {}) is not implemented yet", def.name, a.lib, a.id)),
    };
    Ok(s)
}
