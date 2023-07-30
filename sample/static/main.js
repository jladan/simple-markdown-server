// Change action of directory browser to insert chunks
let dirNav = document.querySelector("#left-pane");
let contentView = document.querySelector("#content-view");

// TODO: This method of updating does not play well with non-html files
dirNav.addEventListener('click', (event) => {
    event.preventDefault();
    console.log('click detected', event.target)
    if (event.target.tagName !== 'A') {
        return;
    }
    fetch(event.target, {
        method: "GET",
        headers: {
            "x-partial": "true",
        },
    }).then((response) => {
        if (!response.ok) {
            throw new Error(`HTTP error, status = ${response.status}`);
        }
        history.pushState({}, 'new page', response.url);
        return response.text()
    }).then((body) => {
        contentView.innerHTML = body;
        hljs.highlightAll();
        MathJax.typeset();
    }).catch((error) => {
        console.log(`Error: ${error.message}`);
    });

}, {capture: true});

// Collapsible directories
let navdirbuttons = document.querySelectorAll(".directory-collapse");

for (let btn of navdirbuttons) {
    btn.addEventListener('click', (event) => {
        event.preventDefault();
        btn.parentElement.classList.toggle("collapsed");
    }, {capture: true});
}

