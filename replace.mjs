import fs from 'fs';

let css = fs.readFileSync('src/landing/LandingPage.css', 'utf-8');

// Replace fonts
css = css.replace(/@import url\("https:\/\/fonts.googleapis.com\/css2\?family=Space\+Grotesk[^"]*"\);\n?/g, '@import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap");\n');
css = css.replace(/"Manrope", "Segoe UI", sans-serif/g, '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Helvetica Neue", sans-serif');
css = css.replace(/"Space Grotesk", "Segoe UI", sans-serif/g, '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Helvetica Neue", sans-serif');

// Replace CSS Custom Properties in :root
const newRootVars = `
  --landing-bg: #060a10;
  --landing-bg-deep: #05070c;
  --landing-surface: #0c121a;
  --landing-surface-strong: #111a25;
  --landing-surface-soft: #0f1620;
  --landing-line: #233142;
  --landing-line-strong: #44607f;
  --landing-text: #fafafa;
  --landing-text-soft: #c9d2df;
  --landing-text-muted: #8a97ab;
  --landing-accent: #79bbff;
  --landing-accent-strong: #e2f0ff;
  --landing-accent-alt: #ffb870;
  --landing-highlight: #e2f0ff;
  --landing-success: #3ed2a2;
  --landing-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
`;
css = css.replace(/:root\s*{[\s\S]*?(?=\s*html\s*{)/, `:root {\n  color-scheme: dark;${newRootVars}}\n\n`);

// Clean up body background
css = css.replace(
  /background:\s*radial-gradient[^;]*linear-gradient[^;]*;/g,
  `background:\n    radial-gradient(circle at 15% 18%, rgba(121, 187, 255, 0.1), transparent 28%),\n    radial-gradient(circle at 88% 14%, rgba(255, 184, 112, 0.08), transparent 24%),\n    linear-gradient(180deg, #08101a 0%, #060a10 48%, #05070c 100%);`
);

// Colors specifically used as rgb/rgba
css = css.replace(/rgba\(139,\s*124,\s*255/g, 'rgba(121, 187, 255');
css = css.replace(/rgba\(111,\s*216,\s*255/g, 'rgba(255, 184, 112');
css = css.replace(/rgba\(197,\s*156,\s*255/g, 'rgba(121, 187, 255');

css = css.replace(/rgba\(169,\s*155,\s*255/g, 'rgba(121, 187, 255'); // glow/shadow
css = css.replace(/rgba\(168,\s*153,\s*255/g, 'rgba(121, 187, 255'); // line-strong

// Primary button gradient
css = css.replace(/linear-gradient\(135deg, var\(--landing-accent\) 0%, #635bff 100%\)/g, 'linear-gradient(135deg, rgba(121, 187, 255, 0.16) 0%, rgba(121, 187, 255, 0.05) 100%)');
css = css.replace(/rgba\(99,\s*91,\s*255/g, 'rgba(121, 187, 255');
css = css.replace(/linear-gradient\(135deg, #7c6cff 0%, #5f6dff 100%\)/g, 'var(--landing-accent)');
// Let's actually adjust primary button to match app style. 
// App style primary button might just be a flat accent blue. Wait, App doesn't have a giant primary button in the CSS above, maybe it's just 'background: var(--accent); color: #060a10;'
css = css.replace(/\.landing-button--primary\s*{[\s\S]*?}/, `.landing-button--primary {
  color: #060a10;
  background: var(--landing-accent);
  box-shadow: 0 4px 14px rgba(121, 187, 255, 0.22);
}
.landing-button--primary small {
  opacity: 0.6;
}
`);

// Adjust cards to match the app 'surface'
// Application surface:
// background: linear-gradient(180deg, rgba(17, 24, 36, 0.84) 0%, rgba(10, 14, 20, 0.92) 100%);
// box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.03), 0 12px 38px rgba(0, 0, 0, 0.2);
css = css.replace(/background:\s*linear-gradient\([^;]*rgba\(17, 20, 34[^;]*;/g, `background:\n    linear-gradient(180deg, rgba(17, 24, 36, 0.84) 0%, rgba(10, 14, 20, 0.92) 100%);`);
css = css.replace(/background:\s*linear-gradient\([^;]*rgba\(15, 17, 29[^;]*;/g, `background:\n    linear-gradient(180deg, rgba(17, 24, 36, 0.84) 0%, rgba(10, 14, 20, 0.92) 100%);`);

// hero-desk app style
css = css.replace(/background:\s*radial-gradient\([^;]*linear-gradient\([^;]*rgba\(13, 16, 28[^;]*;/g, `background:\n    radial-gradient(circle at top right, rgba(255, 184, 112, 0.12), transparent 28%),\n    radial-gradient(circle at left center, rgba(121, 187, 255, 0.12), transparent 34%),\n    linear-gradient(180deg, rgba(17, 24, 36, 0.9) 0%, rgba(10, 14, 20, 0.94) 100%);`);

// feature card hover
css = css.replace(/background: rgba\(255, 255, 255, 0.05\);/g, `background: rgba(121, 187, 255, 0.08);`);

// .hero-desk::after
//     background: linear-gradient(180deg, rgba(255, 255, 255, 0.08), transparent 22%); -> matches surface after.

// remove uppercase from h1
css = css.replace(/letter-spacing: -0\.05em;/g, `letter-spacing: -0.02em;`);

// remove uppercase text from buttons and tracking inside components
css = css.replace(/letter-spacing: 0\.16em;\s*text-transform: uppercase;/g, `letter-spacing: 0.16em;\n  text-transform: uppercase;`);

fs.writeFileSync('src/landing/LandingPage.css', css);
