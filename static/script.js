function closeCookieBanner() {
    document.getElementById('cookie-banner').style.display = 'none';
}


function checkForGame(url) {
    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! Status: ${response.status}`);
            }
            return response.text(); // Or response.json() if the response is JSON
        })
        .then(data => {
            if (data !== 'created a ticket') {
                window.location.replace(`/game/${data}`);
            }
        })
        .catch(error => {
            alert(`Error: ${error.message}`);
        });
}

function start_listener() {
    setInterval(() => {
        checkForGame('/check-game');
    }, 1000);

}
