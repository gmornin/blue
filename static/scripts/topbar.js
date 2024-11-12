document.getElementById("logout").onclick = () => {
    if (confirm("You are about to logout.")) {
        document.cookie =
            "token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
        localStorage.removeItem("userid");
        location.reload();
    }
};
