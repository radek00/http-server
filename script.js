function fetchFiles() {
    fetch('/api/directory?path=/')
        .then(response => response.json())
        .then(data => {
            console.log(data);
        });
}
fetchFiles();