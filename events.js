var worker = {postMessage() {}};
var page, res = [];
window.addEventListener("scroll", function(e) {
    if (window.innerHeight + window.scrollY >= document.body.scrollHeight - 100) {
        page++;
        load(res, page);
        checkLength();
    }
});
function checkLength() {
    if (res && (page + 1) * 100 - 1 >= res.length) {
        $`bottom`.innerHTML = res.length ? "the end" : "";
    }
}
function clearRes() {
    res = null;
    [`res`, `len`, `bottom`].forEach(x => $(x).innerHTML = "");
}
let URLfromQuery = q => window.location.href.split("?")[0] + (q ? "?q=" + encodeURIComponent(q) : "");
function navigate(q, push_state = true, is_search = false) {
    clearRes();
    if (!is_search) $`search`.value = q;
    let new_link = URLfromQuery(q);
    if (push_state) {
        window.history.pushState("", "", new_link);
    } else {
        window.history.replaceState("", "", new_link);
    }
    if (q == "") {
        page = 0;
        return;
    }
    $`bottom`.innerHTML = "loading";
    worker.postMessage({q});
}
let timer;
$`search`.addEventListener("input", function() {
    clearTimeout(timer);
    clearRes();
    $`bottom`.innerHTML = "loading";
    timer = setTimeout(() => {
        navigate(this.value.trim(), false, true);
    }, 200);
});
$`clear`.addEventListener("click", function() {
    $`search`.focus();
    navigate("", false);
});