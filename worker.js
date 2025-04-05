importScripts("dict.js");
onmessage = e => {
    var q = e.data.q;
    var res = search(q);
    postMessage(res);
}
function search(q) {
    let terms = q.split(" ");
    terms = terms.map(term => {
        let [_, operator, query] = term.match(/^([=~!])(.*)/) ?? [];
        if (!operator) return {op: "", orig: term, value: term.toLowerCase()};
        if (["=", "~"].includes(operator)) {
            try {let regex = RegExp(query);}
            catch (e) {return {err: "bad regex"}}
        }
        return {
            op: operator,
            orig: query,
            value: query.toLowerCase()
        };
    });
    let err = terms.find(t => t.err);
    if (err) return err;
    let excluded = terms.filter(t => t.op == "!").map(t => search(t.orig));
    err = excluded.find(e => e.err);
    if (err) return err;
    excluded = new Set(excluded.flat().map(e => e[0]));
    let res = [];
    for (const entry of dict) {
        if (excluded.has(entry)) continue;
        let scores = terms.filter(t => t.op != "order").map(({op, orig, value}) => {
            // 5: head
            if (op == "=" && RegExp("^(?:" + value + ")$").test(entry.head) || value == entry.head) return 7;
            if (entry.head.includes(value) || op == "~" && RegExp(value).test(entry.head)) return 6;
            // 3: body
            if (!op) {
                let sanitized = value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
                if (entry.defs.some(def => RegExp(`^[BCDFGJKJKNPSV] ((is|are) (an? |the |an?/the )?)?${sanitized}(e?s|\\W|$)`, "iu").test(def.body))) return 5;
                if (entry.defs.some(def => def.body.toLowerCase().includes(value))) return 4;
                if (entry.derived && entry.derived.some(deriv => deriv.body.toLowerCase().includes(value))) return 3;
                if (entry.head.includes(value)) return 2;
                if (entry.head.startsWith(value)) return 1;
            }
            // other
            if (op == "!") return 0.1;
        })
        if (scores.some(s => !s)) continue;
        res.push([entry, Math.max(...scores)]);
    }
    return res.sort((a,b) => b[1] - a[1]);
}