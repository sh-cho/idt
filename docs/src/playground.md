# Playground

Try **idt** directly in your browser — no installation required. This playground runs idt compiled to WebAssembly.

<div id="wasm-loading" class="playground-loading">Loading WASM module...</div>

<div id="playground-main" class="playground-container" style="display:none">

<div class="playground-tabs">
    <button class="playground-tab active" data-tab="generate">Generate</button>
    <button class="playground-tab" data-tab="inspect">Inspect</button>
    <button class="playground-tab" data-tab="validate">Validate</button>
</div>

<div id="panel-generate" class="playground-panel active">
    <div class="playground-form">
        <div class="playground-field">
            <label for="gen-type">ID Type</label>
            <select id="gen-type"></select>
        </div>
        <div class="playground-field">
            <label for="gen-count">Count</label>
            <input type="number" id="gen-count" value="1" min="1" max="100">
        </div>
        <button class="playground-btn" id="gen-btn">Generate</button>
    </div>
    <div class="playground-result" id="gen-result"></div>
</div>

<div id="panel-inspect" class="playground-panel">
    <div class="playground-form">
        <div class="playground-field">
            <label for="inspect-input">ID Value</label>
            <input type="text" id="inspect-input" placeholder="Paste an ID here...">
        </div>
        <div class="playground-field">
            <label for="inspect-type-hint">Type Hint</label>
            <select id="inspect-type-hint"></select>
        </div>
        <button class="playground-btn" id="inspect-btn">Inspect</button>
    </div>
    <div class="playground-result" id="inspect-result"></div>
</div>

<div id="panel-validate" class="playground-panel">
    <div class="playground-form">
        <div class="playground-field">
            <label for="validate-input">ID Value</label>
            <input type="text" id="validate-input" placeholder="Paste an ID here...">
        </div>
        <div class="playground-field">
            <label for="validate-type-hint">Type Hint</label>
            <select id="validate-type-hint"></select>
        </div>
        <button class="playground-btn" id="validate-btn">Validate</button>
    </div>
    <div class="playground-result" id="validate-result"></div>
</div>

</div>

<link rel="stylesheet" href="playground/playground.css">
<script type="module" src="playground/playground.js"></script>
