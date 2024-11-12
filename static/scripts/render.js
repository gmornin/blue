let startRender = document.getElementById("render");
let success = document.getElementById("success");
let failed = document.getElementById("failed");
let timer = document.getElementById("timer");
let viewerror = document.getElementById("viewerror");
let reload = document.getElementById("reload");

let params = new URL(window.location.toLocaleString()).searchParams;
let source = params.get("source");
let target = params.get("target");

let interval;
let error;

let timerCounting = 0;
let reloadCountdown = 6;

function getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(";").shift();
}

function getToken() {
    return getCookie("token");
}

viewerror.onclick = () => alert(error);
reload.onclick = window.reload;

function errored(msg) {
    let mins = Math.floor(timerCounting / 60);
    let secs = timerCounting % 60;
    timer.innerText = `Rendering ended in ${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;

    clearInterval(interval);
    error = msg;
    failed.innerText = "Map render failed";
    failed.classList.remove("hide");
    viewerror.classList.remove("hide");
    reload.classList.remove("hide");
}

function completed() {
    let tickComplete = () => {
        reload.innerText = `Reloading in ${--reloadCountdown}`;
        if (reloadCountdown == 0) location.reload();
    };
    tickComplete();
    setInterval(tickComplete, 1000);

    let mins = Math.floor(timerCounting / 60);
    let secs = timerCounting % 60;
    timer.innerText = `Rendering ended in ${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;

    clearInterval(interval);
    success.innerText = "Map render completed";
    success.classList.remove("hide");
    reload.classList.remove("hide");
}

startRender.onclick = () => {
    if (startRender.getAttribute("disabled") == "disabled") return;
    startRender.setAttribute("disabled", "disabled");

    let tick = () => {
        let mins = Math.floor(timerCounting / 60);
        let secs = timerCounting % 60;
        timer.innerText = `Rendering: ${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")} elapsed`;
        timerCounting++;
    };

    tick();
    interval = setInterval(tick, 1000);

    let body = {
        token: getToken(),
        from: source,
        to: target,
        preset: document.getElementById("preset").value,
    };

    let url = "/api/blue/v1/render";
    fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    })
        .then((response) => response.json())
        .then((data) => {
            if (data.type == "error") {
                errored(JSON.stringify(data.kind));
                return;
            }

            completed();
        })
        .catch((error) => errored(JSON.stringify(error)));
};
