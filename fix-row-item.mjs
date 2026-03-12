import fs from 'fs';

let css = fs.readFileSync('src/App.css', 'utf-8');

// Modify row-item to have better flexbox handling
css += `
/* Chat Sidebar Thread Item Overrides */
.chat-sidebar-group .row-item {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
  padding: 12px;
}
.chat-sidebar-group .row-item__main {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.chat-sidebar-group .row-item__title-row {
  display: flex;
  flex-wrap: nowrap;
  gap: 8px;
  justify-content: space-between;
  width: 100%;
}
.chat-sidebar-group .row-item__title {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}
.chat-sidebar-group .soft-badge {
  flex-shrink: 0;
}
.chat-sidebar-group .row-item__meta {
  display: flex;
  justify-content: space-between;
  width: 100%;
  font-size: 0.75rem;
}
`;

fs.writeFileSync('src/App.css', css);
