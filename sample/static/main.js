let navdirbuttons = document.querySelectorAll(".directory-collapse");

for (let btn of navdirbuttons) {
    btn.addEventListener('click', () => {
        btn.parentElement.classList.toggle("collapsed");
    });
}
