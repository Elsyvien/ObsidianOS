const fs = require('fs');

let css = fs.readFileSync('src/App.css', 'utf8');

css = css.replace(/\.chat-transcript {[\s\S]*?}/, \.chat-transcript {
  display: flex !important;
  flex-direction: column;
  gap: 20px;
  min-height: 420px;
  max-height: 68vh;
  overflow-y: auto;
  padding: 12px 16px 12px 4px;
}\);

css = css.replace(/\.chat-message {[\s\S]*?}/, \.chat-message {
  display: flex !important;
  flex-direction: column;
  gap: 8px;
  padding: 16px 20px;
  border-radius: 20px;
  background: transparent;
  max-width: 85%;
}\);

css = css.replace(/\.chat-message--assistant {[\s\S]*?}/, \.chat-message--assistant {
  border: 1px solid rgba(16, 185, 129, 0.15);
  background: rgba(16, 185, 129, 0.04);
  align-self: flex-start;
  border-bottom-left-radius: 4px;
}\);

css = css.replace(/\.chat-message--user {[\s\S]*?}/, \.chat-message--user {
  border: 1px solid rgba(106, 175, 255, 0.15);
  background: rgba(106, 175, 255, 0.08);
  align-self: flex-end;
  border-bottom-right-radius: 4px;
}\);

if (!css.includes('.chat-message--user')) {
  css += \\n\n.chat-message--user {
  border: 1px solid rgba(106, 175, 255, 0.15);
  background: rgba(106, 175, 255, 0.08);
  align-self: flex-end;
  border-bottom-right-radius: 4px;
}\n\;
}

css = css.replace(/\.chat-message--streaming {[\s\S]*?}/, \.chat-message--streaming {
  border: 1px solid rgba(106, 175, 255, 0.28);
  background: linear-gradient(180deg, rgba(106, 175, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
  align-self: flex-start;
  border-bottom-left-radius: 4px;
  max-width: auto;
  align-items: center;
  padding: 12px 20px;
}\);

css = css.replace(/\.chat-sidebar-group {[\s\S]*?}/, \.chat-sidebar-group {
  display: grid;
  gap: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--line);
}\);

css = css.replace(/\.row-item__main {/g, \.row-item__main {
  width: 100%;\);

fs.writeFileSync('src/App.css', css);
console.log("Patched css");
