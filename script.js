const pathElem  = document.querySelector('.path-wrapper div');
const upButton = document.getElementById('up-button');
let currentPathElem;
let currentPaths;

function renderPath(pathArray) {
    pathElem.innerHTML = '';
    if (pathArray.length > 1) upButton.removeAttribute('disabled');
    pathArray.forEach((path) => {
        const pathLink = document.createElement('a');
        pathLink.href = '#';
        pathLink.textContent = path.part_name === "/" ? "/root" : path.part_name;

        pathLink.onclick = (event) => {
            event.preventDefault();
            currentPathElem.classList.toggle('current');
            pathLink.classList.toggle('current');
            currentPathElem = pathLink;
            fetchPath(path.full_path);
        };
        pathElem.appendChild(pathLink);
    });
    currentPathElem = pathElem.lastChild;
    currentPathElem.classList.add('current');
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
                fetchPath(file.path);
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
    const previousPath = currentPaths[currentPaths.length - 2];
    if (previousPath.part_name === "/") {
        upButton.setAttribute('disabled', 'true');
    }
    console.log(previousPath);
    fetchPath(previousPath.full_path);
}

function fetchPath(path = "./") {
    fetch(`/api/path?path=${path}`).then(resp => resp.json()).then(paths => {
        console.log(paths);
        currentPaths = paths;
        renderPath(paths);
        renderFileTree(paths[paths.length -1].full_path);
    });
}

fetchPath();