const $ = x => document.getElementById(x);
function mkel(tag, props, children) {
    const element = document.createElement(tag);
    Object.assign(element, props);
    for (const child of children) {
        if (child) {
            element.append(child);
        }
    }
    return element;
}
function htmlify(json) {
    const entry = mkel("div", {"className": "entry"}, [
        mkel("dt", null, [
            mkel("a", {"href": "?q=" + json.head, "className": "head"
            }, [
                mkel("b", null, [json.head])
            ]),
            " ",
            json.affix ? mkel("i", {"className": "affix"}, [
                json.affix.join(" ")
            ]) : null,
            " ",
            json.etym ? mkel("span", {"className": "etym"}, [
                "← ", json.etym
            ]) : null
        ]),
        json.defs ? json.defs.map(def => {
            return [
                mkel("i", {}, [def.pos + ". "]),
                def.frame ? mkel("code", {}, ["[" + def.frame + "]"]) : null,
                " ",
                def.body,
                mkel("br", {}, [])
            ];
        }) : null,
        json.derivs ? json.derivs.map(deriv => {
            return [
                mkel("b", {}, [deriv.head]),
                " ",
                mkel("i", {}, [deriv.pos + ". "]),
                deriv.body,
                mkel("br", {}, [])
            ];
        }) : null,
        json.used_in ? mkel("details", {}, [
            mkel("summary", {}, "used in"),
            json.used_in.map(u => {
                return [
                    mkel("a", {"href": "?q=" + u}, [u]),
                    ", "
                ];
            })
        ].flat(Infinity).slice(0, -1)) : null
    ].flat(Infinity));
    return entry;
}
function load(res, page) {
    const start = page * 100;
    const end = (page + 1) * 100;
    var nodes = [];
    for (var i = start; i < end; i++) {
        if (res[i]) {
            nodes.push(htmlify(res[i][0]));
        }
    }
    $`res`.append(...nodes);
}