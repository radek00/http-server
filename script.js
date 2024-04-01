let previousPath = '';

function renderUpButton() {
    const button  = document.getElementById('up-button');
    button.onclick = () => {
        renderFileTree(previousPath);
    }
    button.innerText = `UP - ${previousPath}`;
}

async function fetchFiles(path = "") {
    const files = await fetch(`/api/directory?path=${path}`);
    return await files.json();
}

async function renderFileTree(path = "") {
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
                previousPath = `${file.path.split('/').slice(0, -1).join('/')}/`;
                renderUpButton();
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

renderUpButton();
renderFileTree(previousPath);