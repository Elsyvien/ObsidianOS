import { ArrowRight, Download, Github, MonitorDown, NotebookTabs } from "lucide-react";
import { BrandMark } from "../components/BrandMark";
import {
  FAQ_ITEMS,
  FEATURE_CARDS,
  FOOTER_LINKS,
  HERO_CTAS,
  HERO_PILLARS,
  LANDING_NAV,
  SHOWCASE_ITEMS,
  WORKFLOW_STEPS,
} from "./content";
import "./LandingPage.css";

export function LandingPage() {
  return (
    <div className="premium-layout" id="top">
      <nav className="premium-nav">
        <div className="premium-nav__container">
          <a className="premium-nav__brand" href="#top">
            <BrandMark className="premium-nav__logo" />
            <span>ObsidianOS</span>
          </a>

          <div className="premium-nav__links">
            {LANDING_NAV.map((item) => (
              <a key={item.id} href={`#${item.id}`}>
                {item.label}
              </a>
            ))}
          </div>

          <div className="premium-nav__actions">
            <a className="premium-button premium-button--small" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
              Download
            </a>
          </div>
        </div>
      </nav>

      <main>
        <section className="premium-hero">
          <div className="premium-hero__bg">
            <div className="glow glow--left" />
            <div className="glow glow--right" />
          </div>

          <div className="premium-hero__content">
            <div className="premium-hero__pill">
              <span className="premium-hero__pill-dot" />
              Windows Desktop Exclusive
            </div>
            
            <h1 className="premium-hero__title">
              Your vault is now <br/> an exam system.
            </h1>
            
            <p className="premium-hero__subtitle">
              ObsidianOS scans course folders, builds a local graph, surfaces weak links, extracts formulas, generates flashcards, and writes revision notes back.
            </p>

            <div className="premium-hero__actions">
              <a className="premium-button premium-button--primary" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
                <Download size={18} strokeWidth={2} />
                <span>Get obsidianOS</span>
              </a>
              <a className="premium-button premium-button--secondary" href={HERO_CTAS[1].href} target="_blank" rel="noreferrer">
                <Github size={18} strokeWidth={2} />
                <span>Source code</span>
              </a>
            </div>

            <div className="premium-hero__metrics">
              {HERO_PILLARS.map((pillar) => (
                <span key={pillar}>{pillar}</span>
              ))}
            </div>
          </div>
        </section>

        <section className="premium-section" id="features">
          <div className="premium-section__header">
            <h2>Designed for recall.</h2>
            <p>An intelligent study system built around note structure and exam pressure.</p>
          </div>

          <div className="bento-grid">
            {FEATURE_CARDS.map((feature, idx) => (
              <article key={feature.title} className={`premium-card bento-item--${idx}`}>
                <div className="premium-card__icon">
                   {idx === 0 ? <MonitorDown size={24} /> : idx === 1 ? <NotebookTabs size={24} /> : <ArrowRight size={24} />}
                </div>
                <h3>{feature.title}</h3>
                <p>{feature.description}</p>
                <ul className="premium-card__list">
                  {feature.bullets.slice(0, 3).map((bullet) => (
                    <li key={bullet}>{bullet}</li>
                  ))}
                </ul>
              </article>
            ))}
          </div>
        </section>

        <section className="premium-section premium-section--alternate" id="workflow">
          <div className="premium-section__header">
            <h2>Seamless workflow.</h2>
            <p>Start from a vault, end with a repeatable revision loop.</p>
          </div>

          <div className="workflow-track">
            {WORKFLOW_STEPS.map((step, idx) => (
              <div key={step.id} className="workflow-step premium-card">
                <div className="workflow-step__number">0{idx + 1}</div>
                <div className="workflow-step__content">
                  <h3>{step.title}</h3>
                  <p>{step.description}</p>
                  <span className="workflow-step__outcome">{step.outcome}</span>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="premium-section" id="showcase">
          <div className="premium-section__header">
            <h2>The Workspace.</h2>
            <p>Static captures from the current browser-preview UI.</p>
          </div>

          <div className="showcase-gallery">
            {SHOWCASE_ITEMS.map((item) => (
              <div key={item.title} className="showcase-item premium-card">
                <div className="showcase-item__image">
                  <img alt={item.alt} src={item.imageSrc} />
                </div>
                <div className="showcase-item__caption">
                  <h4>{item.title}</h4>
                  <p>{item.description}</p>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="premium-section" id="scope">
          <div className="premium-section__header">
            <h2>Under the hood.</h2>
            <p>What it is, and what it isn't.</p>
          </div>

          <div className="bento-grid bento-grid--faq">
            <div className="premium-card bento-item--hero">
              <span className="premium-card__eyebrow">Architecture</span>
              <h3>Marketing and download surface</h3>
              <p>
                The browser build exists so the project can live on GitHub Pages under <code>/ObsidianOS/</code>.
                Downloads, docs, and screenshots live here. Real vault access and file writes stay in the desktop app.
              </p>
            </div>
            {FAQ_ITEMS.map((item) => (
              <div key={item.question} className="premium-card">
                <h3>{item.question}</h3>
                <p className="text-secondary">{item.answer}</p>
              </div>
            ))}
          </div>
        </section>

        <section className="premium-cta">
          <div className="premium-cta__content">
            <h2>Ready to upgrade your study system?</h2>
            <p>The GitHub Releases page is the main delivery point for the actual desktop build.</p>
            <div className="premium-cta__actions">
              <a className="premium-button premium-button--white" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
                Download for Windows
              </a>
            </div>
          </div>
        </section>
      </main>

      <footer className="premium-footer">
        <div className="premium-footer__content">
          <div className="premium-footer__brand">
            <BrandMark className="premium-footer__logo" />
            <span className="premium-footer__copyright">
              © {new Date().getFullYear()} ObsidianOS
            </span>
          </div>
          <div className="premium-footer__links">
            {FOOTER_LINKS.map((link) => (
              <a key={link.label} href={link.href} target="_blank" rel="noreferrer">
                {link.label}
              </a>
            ))}
          </div>
        </div>
      </footer>
    </div>
  );
}
