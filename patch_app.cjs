
const fs = require('fs');
let css = fs.readFileSync('src/App.css', 'utf8');

// Use precise replacements without dynamic capturing
css = css.replace('.formula-workspace {\n  grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);\n  gap: 18px;\n  align-items: stretch;\n  height: clamp(42rem, calc(100vh - 11.5rem), 68rem);\n  max-height: calc(100vh - 11.5rem);\n  overflow: hidden;\n}', \.formula-workspace {
  grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
  gap: 18px;
  align-items: stretch;
}\);

css = css.replace('.formula-workspace__rail {\n  display: grid;\n  grid-template-rows: auto auto minmax(0, 1fr);\n  min-width: 0;\n  min-height: 0;\n}', \.formula-workspace__rail {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  min-width: 0;
  height: 0;
  min-height: 100%;
}\);

css = css.replace('.formula-workspace__main {\n  display: grid;\n  gap: 18px;\n  min-width: 0;\n  min-height: 0;\n  align-content: start;\n  overflow: auto;\n  padding-right: 4px;\n}', \.formula-workspace__main {
  display: grid;
  gap: 18px;
  min-width: 0;
  min-height: 0;
  align-content: start;
}\);

fs.writeFileSync('src/App.css', css);
console.log('App.css patched');

