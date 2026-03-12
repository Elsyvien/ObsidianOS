const fs = require('fs');
let css = fs.readFileSync('src/App.css', 'utf-8');

// Also remove duplicate .chat-thread-group .row-item rules
const parts = css.split('/* Chat Sidebar Thread Item Overrides */');
css = parts[0] + '/* Chat Sidebar Thread Item Overrides */\n' + `
.chat-layout__rail {
  min-width: 0;
}
.chat-sidebar-group, .chat-thread-group {
  min-width: 0;
}
.chat-thread-group .row-item {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
  padding: 12px;
  min-width: 0;
}
.chat-thread-group .row-item__main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.chat-thread-group .row-item__title-row {
  display: flex;
  flex-wrap: nowrap;
  gap: 8px;
  justify-content: space-between;
  width: 100%;
  min-width: 0;
}
.chat-thread-group .row-item__title {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}
.chat-thread-group .soft-badge {
  flex-shrink: 0;
}
.chat-thread-group .row-item__meta {
  display: flex;
  justify-content: space-between;
  width: 100%;
  font-size: 0.75rem;
}
`;

fs.writeFileSync('src/App.css', css);
