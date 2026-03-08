import init, { generate, inspect, validate, detect, list_types } from './pkg/idt_wasm.js';

let wasmReady = false;

async function initWasm() {
    try {
        await init();
        wasmReady = true;
        populateTypeDropdowns();
        document.getElementById('wasm-loading').style.display = 'none';
        document.getElementById('playground-main').style.display = 'block';
    } catch (e) {
        document.getElementById('wasm-loading').textContent = 'Failed to load WASM module: ' + e.message;
    }
}

function populateTypeDropdowns() {
    const types = JSON.parse(list_types());
    const genSelect = document.getElementById('gen-type');
    const hintSelect = document.getElementById('inspect-type-hint');
    const valHintSelect = document.getElementById('validate-type-hint');

    types.forEach(t => {
        const opt = document.createElement('option');
        opt.value = t.name;
        opt.textContent = `${t.name} — ${t.description}`;
        genSelect.appendChild(opt);
    });

    // For hint dropdowns, add an "auto-detect" option first
    [hintSelect, valHintSelect].forEach(select => {
        const allTypes = [
            { name: '', description: 'Auto-detect' },
            ...types,
        ];
        allTypes.forEach(t => {
            const opt = document.createElement('option');
            opt.value = t.name;
            opt.textContent = t.name ? `${t.name} — ${t.description}` : 'Auto-detect';
            select.appendChild(opt);
        });
    });
}

function showResult(containerId, html) {
    document.getElementById(containerId).innerHTML = html;
}

function showError(containerId, msg) {
    showResult(containerId, `<div class="playground-error">${escapeHtml(msg)}</div>`);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function renderTable(obj, skipKeys = []) {
    let rows = '';
    for (const [key, value] of Object.entries(obj)) {
        if (skipKeys.includes(key) || value === null || value === undefined) continue;
        let display;
        if (typeof value === 'object') {
            display = escapeHtml(JSON.stringify(value, null, 2));
        } else {
            display = escapeHtml(String(value));
        }
        rows += `<tr><th>${escapeHtml(key)}</th><td>${display}</td></tr>`;
    }
    return `<table class="playground-table">${rows}</table>`;
}

// Tab switching
function setupTabs() {
    document.querySelectorAll('.playground-tab').forEach(tab => {
        tab.addEventListener('click', () => {
            document.querySelectorAll('.playground-tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.playground-panel').forEach(p => p.classList.remove('active'));
            tab.classList.add('active');
            document.getElementById(`panel-${tab.dataset.tab}`).classList.add('active');
        });
    });
}

// Generate
function setupGenerate() {
    document.getElementById('gen-btn').addEventListener('click', () => {
        if (!wasmReady) return;
        const idType = document.getElementById('gen-type').value;
        const count = parseInt(document.getElementById('gen-count').value, 10) || 1;
        try {
            const ids = JSON.parse(generate(idType, Math.min(count, 100)));
            const items = ids.map(id => `<li title="Click to copy">${escapeHtml(id)}</li>`).join('');
            showResult('gen-result',
                `<ul class="result-ids">${items}</ul><div class="copy-hint">Click an ID to copy</div>`
            );
            // Copy on click
            document.querySelectorAll('#gen-result .result-ids li').forEach(li => {
                li.addEventListener('click', () => {
                    navigator.clipboard.writeText(li.textContent);
                    li.style.background = 'var(--sidebar-active)';
                    li.style.color = 'var(--sidebar-bg)';
                    setTimeout(() => { li.style.background = ''; li.style.color = ''; }, 300);
                });
            });
        } catch (e) {
            showError('gen-result', e.message || String(e));
        }
    });
}

// Inspect
function setupInspect() {
    document.getElementById('inspect-btn').addEventListener('click', () => {
        if (!wasmReady) return;
        const idValue = document.getElementById('inspect-input').value.trim();
        if (!idValue) return;
        const typeHint = document.getElementById('inspect-type-hint').value || undefined;
        try {
            const result = JSON.parse(inspect(idValue, typeHint));
            showResult('inspect-result', renderTable(result));
        } catch (e) {
            showError('inspect-result', e.message || String(e));
        }
    });
}

// Validate
function setupValidate() {
    document.getElementById('validate-btn').addEventListener('click', () => {
        if (!wasmReady) return;
        const idValue = document.getElementById('validate-input').value.trim();
        if (!idValue) return;
        const typeHint = document.getElementById('validate-type-hint').value || undefined;
        try {
            const result = JSON.parse(validate(idValue, typeHint));
            showResult('validate-result', renderTable(result));
        } catch (e) {
            showError('validate-result', e.message || String(e));
        }
    });
}

// Enter key support
function setupEnterKey() {
    document.getElementById('inspect-input').addEventListener('keydown', e => {
        if (e.key === 'Enter') document.getElementById('inspect-btn').click();
    });
    document.getElementById('validate-input').addEventListener('keydown', e => {
        if (e.key === 'Enter') document.getElementById('validate-btn').click();
    });
}

document.addEventListener('DOMContentLoaded', () => {
    setupTabs();
    setupGenerate();
    setupInspect();
    setupValidate();
    setupEnterKey();
    initWasm();
});
