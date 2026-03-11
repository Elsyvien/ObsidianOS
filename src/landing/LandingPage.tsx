import { ArrowRight, Download, ExternalLink, Github, MonitorDown, NotebookTabs } from "lucide-react";
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

const brandLogoUrl = `${import.meta.env.BASE_URL}obsidianos-logo.svg`;

export function LandingPage() {
  return (
    <div className="landing-shell" id="top">
      <header className="landing-topbar">
        <a className="landing-brand" href="#top">
          <img alt="ObsidianOS logo" className="landing-brand__image" src={brandLogoUrl} />
          <span className="landing-brand__copy">
            <strong>ObsidianOS</strong>
            <span>Desktop study workstation for Obsidian vaults</span>
          </span>
        </a>

        <nav aria-label="Primary" className="landing-nav">
          {LANDING_NAV.map((item) => (
            <a key={item.id} href={`#${item.id}`}>
              {item.label}
            </a>
          ))}
        </nav>

        <a className="landing-inline-cta" href={HERO_CTAS[0].href} rel="noreferrer" target="_blank">
          <Download size={16} strokeWidth={1.8} />
          Download
        </a>
      </header>

      <main className="landing-main">
        <section className="landing-hero">
          <div className="landing-hero__copy">
            <span className="landing-kicker">Study workstation</span>
            <h1>Turn an Obsidian vault into an exam system, not just a note archive.</h1>
            <p className="landing-lead">
              ObsidianOS scans course folders, builds a local graph, surfaces weak links, extracts formulas,
              generates flashcards, and writes revision notes back into the vault. This website is the product
              page. The app itself stays a Windows desktop client.
            </p>

            <div className="landing-pillars" aria-label="Product highlights">
              {HERO_PILLARS.map((pillar) => (
                <span key={pillar}>{pillar}</span>
              ))}
            </div>

            <div className="landing-cta-row">
              {HERO_CTAS.map((cta) => (
                <a
                  key={cta.label}
                  className={`landing-button landing-button--${cta.variant}`}
                  href={cta.href}
                  rel="noreferrer"
                  target="_blank"
                >
                  <span>{cta.label}</span>
                  {cta.variant === "primary" ? <ArrowRight size={18} strokeWidth={1.8} /> : <ExternalLink size={16} strokeWidth={1.8} />}
                  <small>{cta.note}</small>
                </a>
              ))}
            </div>
          </div>

          <aside className="landing-hero__panel" aria-label="Product summary">
            <div className="hero-desk hero-desk--large">
              <div className="hero-desk__header">
                <span>Current desktop loop</span>
                <strong>ObsidianOS</strong>
              </div>

              <div className="hero-desk__metrics">
                <MetricCard label="Scan pass" value="Markdown to graph" />
                <MetricCard label="Weak notes" value="Graph gaps surfaced" />
                <MetricCard label="Outputs" value="Flashcards and revision" />
              </div>

              <div className="hero-desk__rail">
                <div className="hero-desk__note">
                  <span>Vault</span>
                  <strong>One connected workspace</strong>
                  <p>Top-level folders become course spaces. The desktop app stays in charge of file access and writes.</p>
                </div>
                <div className="hero-desk__stack">
                  <span>Prep loop</span>
                  <ol>
                    <li>Scan the vault</li>
                    <li>Review weak links</li>
                    <li>Generate study outputs</li>
                    <li>Run exam prep</li>
                  </ol>
                </div>
              </div>
            </div>

            <div className="landing-proof">
              <div>
                <MonitorDown size={18} strokeWidth={1.7} />
                <strong>Windows desktop</strong>
                <span>Website for marketing. App for work.</span>
              </div>
              <div>
                <NotebookTabs size={18} strokeWidth={1.7} />
                <strong>Vault-native outputs</strong>
                <span>Revision notes and flashcards stay tied to the source vault.</span>
              </div>
            </div>
          </aside>
        </section>

        <section className="landing-section" id="features">
          <SectionHeading
            eyebrow="Feature overview"
            title="A study system built around note structure, recall, and exam pressure."
            description="The public page stays static, but the product story is grounded in the actual desktop features already in this repository."
          />
          <div className="feature-grid">
            {FEATURE_CARDS.map((feature) => (
              <article key={feature.title} className="feature-card">
                <span>{feature.eyebrow}</span>
                <h2>{feature.title}</h2>
                <p>{feature.description}</p>
                <ul>
                  {feature.bullets.map((bullet) => (
                    <li key={bullet}>{bullet}</li>
                  ))}
                </ul>
              </article>
            ))}
          </div>
        </section>

        <section className="landing-section landing-section--workflow" id="workflow">
          <SectionHeading
            eyebrow="Workflow"
            title="Start from a vault, end with a repeatable revision loop."
            description="The sequence mirrors the desktop product flow instead of pretending there is a web app behind this page."
          />
          <div className="workflow-list">
            {WORKFLOW_STEPS.map((step) => (
              <article key={step.id} className="workflow-card">
                <div className="workflow-card__id">
                  <span>{step.label}</span>
                  <strong>{step.id}</strong>
                </div>
                <div className="workflow-card__body">
                  <h2>{step.title}</h2>
                  <p>{step.description}</p>
                  <small>{step.outcome}</small>
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className="landing-section" id="showcase">
          <SectionHeading
            eyebrow="Showcase"
            title="Static captures from the current browser-preview UI."
            description="These are framed showcase slices of the real mock interface, used here as screenshots rather than as the public product experience."
          />
          <div className="showcase-grid">
            {SHOWCASE_ITEMS.map((item) => (
              <article key={item.title} className="showcase-card">
                <div className="showcase-card__frame">
                  <img alt={item.alt} src={item.imageSrc} />
                </div>
                <div className="showcase-card__copy">
                  <span>{item.imageLabel}</span>
                  <h2>{item.title}</h2>
                  <p>{item.description}</p>
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className="landing-section landing-section--scope" id="scope">
          <SectionHeading
            eyebrow="Current scope"
            title="The README and the page say the same thing."
            description="Keep the claims tight: this is a Windows desktop study tool for Obsidian workflows, not a hosted browser application."
          />
          <div className="scope-layout">
            <div className="scope-card">
              <span>What this page is</span>
              <h2>Marketing and download surface</h2>
              <p>
                The browser build exists so the project can live on GitHub Pages under <code>/ObsidianOS/</code>.
                Downloads, docs, and screenshots live here. Real vault access and file writes stay in the desktop app.
              </p>
            </div>

            <div className="faq-list">
              {FAQ_ITEMS.map((item) => (
                <article key={item.question} className="faq-card">
                  <h2>{item.question}</h2>
                  <p>{item.answer}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section className="landing-download" id="download">
          <div className="landing-download__copy">
            <span>Download</span>
            <h2>Use the landing page in the browser. Use ObsidianOS on Windows.</h2>
            <p>
              The GitHub Pages deployment stays static and subpath-safe. The GitHub Releases page stays the main
              delivery point for the actual desktop build.
            </p>
          </div>

          <div className="landing-download__actions">
            <a className="landing-button landing-button--primary" href={HERO_CTAS[0].href} rel="noreferrer" target="_blank">
              <span>Open GitHub Releases</span>
              <ArrowRight size={18} strokeWidth={1.8} />
              <small>Windows download path</small>
            </a>
            <a className="landing-button landing-button--secondary" href={HERO_CTAS[1].href} rel="noreferrer" target="_blank">
              <span>Browse the source</span>
              <Github size={16} strokeWidth={1.8} />
              <small>Repo, issues, roadmap</small>
            </a>
          </div>
        </section>
      </main>

      <footer className="landing-footer">
        <div className="landing-footer__brand">
          <BrandMark className="landing-footer__mark" />
          <p>ObsidianOS keeps the browser build as a product page and the Tauri runtime as the actual workspace.</p>
        </div>
        <div className="landing-footer__links">
          {FOOTER_LINKS.map((link) => (
            <a key={link.label} href={link.href} rel="noreferrer" target="_blank">
              {link.label}
            </a>
          ))}
        </div>
      </footer>
    </div>
  );
}

function SectionHeading({
  eyebrow,
  title,
  description,
}: {
  eyebrow: string;
  title: string;
  description: string;
}) {
  return (
    <div className="section-heading">
      <span>{eyebrow}</span>
      <h2>{title}</h2>
      <p>{description}</p>
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
