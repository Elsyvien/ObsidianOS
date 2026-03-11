import fs from 'fs';

let css = fs.readFileSync('src/landing/LandingPage.css', 'utf-8');

// Replace border-radii to be tighter
css = css.replace(/border-radius:\s*34px;/g, 'border-radius: 16px;');
css = css.replace(/border-radius:\s*28px;/g, 'border-radius: 16px;');
css = css.replace(/border-radius:\s*26px;/g, 'border-radius: 16px;');
css = css.replace(/border-radius:\s*24px;/g, 'border-radius: 16px;');
css = css.replace(/border-radius:\s*22px;/g, 'border-radius: 12px;');
css = css.replace(/border-radius:\s*18px;/g, 'border-radius: 12px;');

// App.css uses very specific borders:
// 1px solid var(--line)
css = css.replace(/border:\s*1px\s*solid\s*var\(--landing-line\);/g, `border: 1px solid var(--landing-line);
  box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.03), 0 12px 38px rgba(0, 0, 0, 0.2);`);

// App metric style: 
// background: linear-gradient(180deg, rgba(121, 187, 255, 0.08) 0%, rgba(255, 255, 255, 0.015) 100%);
// Let's replace simple backgrounds with app-like ones
css = css.replace(/background:\s*rgba\(255, 255, 255, 0\.04\);/g, 'background: linear-gradient(180deg, rgba(121, 187, 255, 0.04) 0%, rgba(255, 255, 255, 0.01) 100%);');
css = css.replace(/background:\s*rgba\(255, 255, 255, 0\.03\);/g, 'background: linear-gradient(180deg, rgba(121, 187, 255, 0.04) 0%, rgba(255, 255, 255, 0.01) 100%);');
css = css.replace(/background:\s*rgba\(255, 255, 255, 0\.035\);/g, 'background: linear-gradient(180deg, rgba(121, 187, 255, 0.04) 0%, rgba(255, 255, 255, 0.01) 100%);');

// Update font sizes/weights to App standards
// In App: headings are typically font-weight: 500 or 600, not 700 Space Grotesk.
css = css.replace(/font-weight:\s*700;/g, 'font-weight: 600;');
css = css.replace(/font-weight:\s*800;/g, 'font-weight: 600;'); // kickers, eyebrows

fs.writeFileSync('src/landing/LandingPage.css', css);
