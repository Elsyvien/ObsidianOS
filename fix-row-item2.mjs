import fs from 'fs';
let css = fs.readFileSync('src/App.css', 'utf-8');
css = css.replace(/\.chat-sidebar-group/g, '.chat-thread-group');
fs.writeFileSync('src/App.css', css);
