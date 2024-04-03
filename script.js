let currentPath = '';

function renderPath(path = "./") {
    const pathElem  = document.querySelector('.path-wrapper p');
    pathElem.innerHTML = path;
}

async function fetchFiles(path = "./") {
    const files = await fetch(`/api/directory?path=${path}`);
    return await files.json();
}

async function renderFileTree(path = "./") {
    const tbody = document.querySelector('tbody');
    tbody.innerHTML = '';
    const files = await fetchFiles(path);

    files.forEach(file => {
        const tr = document.createElement('tr');
        const name = document.createElement('td');
        const fileLink = document.createElement('a');
        if (file.file_type === 'Directory') {
            fileLink.href = '#';
            fileLink.textContent = `${file.name}/`;
            fileLink.onclick = (event) => {
                event.preventDefault();
                currentPath = file.path;
                renderPath(file.path);
                renderFileTree(file.path);
            }
        } else {
            fileLink.href = `api/files?path=${file.path}`;
            fileLink.textContent = file.name;
        }
        name.appendChild(fileLink);
        const lastModified = document.createElement('td');
        const size = document.createElement('td');
        lastModified.textContent = file.last_modified;
        size.textContent = file.size;
        tr.appendChild(name);
        tr.appendChild(lastModified);
        tr.appendChild(size);
        tbody.appendChild(tr);
    });
};

function onUpClick() {
    const pathArray = currentPath.split('/');
    pathArray.pop();
    const path = pathArray.join('/');
    currentPath = path;
    console.log(path);
    renderPath(path);
    renderFileTree(path);
}

fetch("/api/path").then(resp => resp.text()).then(text => {
    console.log(text);
    currentPath = text;
    renderPath(text);
    renderFileTree(currentPath);
});