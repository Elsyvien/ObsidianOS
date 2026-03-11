import { useState } from "react";
import { Download, Github, ChevronRight, BookOpen, BrainCircuit, Sigma, ClipboardCheck, PenLine, BarChart3 } from "lucide-react";
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

const FEATURE_ICONS = [BarChart3, BookOpen, BrainCircuit, PenLine, Sigma, ClipboardCheck];

function TermsOfService({ onBack }: { onBack: () => void }) {
  return (
    <div className="tos">
      <div className="tos__inner">
        <button className="tos__back" onClick={onBack} type="button">← Back to home</button>
        <h1>Terms of Service</h1>
        <p className="tos__date">Last updated — March 2026</p>

        <h2>1. Acceptance of Terms</h2>
        <p>By downloading, installing, or using ObsidianOS ("the Software"), you agree to these Terms. If you do not agree, do not use the Software.</p>

        <h2>2. Description of Service</h2>
        <p>ObsidianOS is a local, desktop study workstation built with Tauri. It reads Obsidian vault folders, indexes markdown notes, and provides study tools such as flashcard generation, formula extraction, and exam simulation. All processing happens on your device.</p>

        <h2>3. Data &amp; Privacy</h2>
        <p>Your vault data stays on your machine. ObsidianOS does not transmit vault contents, personal data, or credentials to external servers. The only exception is when you explicitly configure a third-party AI API endpoint (e.g. OpenRouter or OpenAI-compatible) for optional note enrichment — in which case, selected note excerpts are sent to that endpoint under your own API key.</p>

        <h2>4. User Responsibilities</h2>
        <ul>
          <li>Back up your Obsidian vault before use.</li>
          <li>Keep API keys stored in the app's local configuration secure.</li>
          <li>Comply with the terms of any third-party AI provider you configure.</li>
        </ul>

        <h2>5. Intellectual Property</h2>
        <p>ObsidianOS is open-source software. You retain full ownership of all vault content, generated flashcards, revision notes, and exam outputs produced by the Software.</p>

        <h2>6. Disclaimer of Warranties</h2>
        <p>THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES, OR OTHER LIABILITY ARISING FROM THE USE OF THE SOFTWARE.</p>

        <h2>7. Changes</h2>
        <p>These terms may be updated from time to time. Continued use of the Software after changes constitutes acceptance.</p>
      </div>
    </div>
  );
}

export function LandingPage() {
  const [page, setPage] = useState<"home" | "tos">("home");

  const goHome = () => { setPage("home"); window.scrollTo({ top: 0, behavior: "smooth" }); };
  const goTos = () => { setPage("tos"); window.scrollTo({ top: 0, behavior: "smooth" }); };

  if (page === "tos") {
    return (
      <div className="lp">
        <NavBar onBrandClick={goHome} />
        <TermsOfService onBack={goHome} />
        <Footer onTos={goTos} />
      </div>
    );
  }

  return (
    <div className="lp" id="top">
      <NavBar onBrandClick={goHome} />

      {/* ── Hero ───────────────────────────────────── */}
      <section className="hero">
        <p className="hero__badge">Cross-platform · Built with Tauri</p>
        <h1 className="hero__h1">
          Your vault is now<br />an exam system.
        </h1>
        <p className="hero__sub">
          ObsidianOS scans course folders, builds a local graph, surfaces weak links, extracts formulas,
          generates flashcards, and writes revision notes back into the vault.
        </p>
        <div className="hero__actions">
          <a className="btn btn--white" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
            <Download size={18} /> Download
          </a>
          <a className="btn btn--outline" href={HERO_CTAS[1].href} target="_blank" rel="noreferrer">
            <Github size={18} /> Source Code
          </a>
        </div>
        <div className="hero__pills">
          {HERO_PILLARS.map((p) => <span key={p} className="pill">{p}</span>)}
        </div>
      </section>

      {/* ── Features ───────────────────────────────── */}
      <section className="sec" id="features">
        <header className="sec__head">
          <p className="sec__kicker">Features</p>
          <h2 className="sec__h2">Everything you need for exam season.</h2>
          <p className="sec__sub">Built around note structure, recall, and real exam pressure.</p>
        </header>

        <div className="grid grid--3">
          {FEATURE_CARDS.map((f, i) => {
            const Icon = FEATURE_ICONS[i % FEATURE_ICONS.length];
            return (
              <article key={f.title} className="card">
                <div className="card__icon"><Icon size={22} strokeWidth={1.6} /></div>
                <p className="card__eyebrow">{f.eyebrow}</p>
                <h3 className="card__h3">{f.title}</h3>
                <p className="card__body">{f.description}</p>
                <ul className="card__bullets">
                  {f.bullets.map((b) => <li key={b}>{b}</li>)}
                </ul>
              </article>
            );
          })}
        </div>
      </section>

      {/* ── Workflow ───────────────────────────────── */}
      <section className="sec sec--alt" id="workflow">
        <header className="sec__head">
          <p className="sec__kicker">Workflow</p>
          <h2 className="sec__h2">From vault to revision loop.</h2>
          <p className="sec__sub">Five steps. No cloud. No subscriptions.</p>
        </header>

        <div className="steps">
          {WORKFLOW_STEPS.map((s, i) => (
            <div key={s.id} className="step">
              <div className="step__num">
                <span>{String(i + 1).padStart(2, "0")}</span>
                {i < WORKFLOW_STEPS.length - 1 && <div className="step__line" />}
              </div>
              <div className="step__body">
                <p className="step__label">{s.label}</p>
                <h3 className="step__h3">{s.title}</h3>
                <p className="step__desc">{s.description}</p>
                <p className="step__outcome"><ChevronRight size={14} /> {s.outcome}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ── Showcase ───────────────────────────────── */}
      <section className="sec" id="showcase">
        <header className="sec__head">
          <p className="sec__kicker">Showcase</p>
          <h2 className="sec__h2">See it in action.</h2>
          <p className="sec__sub">Static captures from the current desktop UI.</p>
        </header>

        <div className="grid grid--3">
          {SHOWCASE_ITEMS.map((item) => (
            <div key={item.title} className="showcase">
              <div className="showcase__img">
                <img alt={item.alt} src={item.imageSrc} loading="lazy" />
              </div>
              <div className="showcase__text">
                <span className="showcase__label">{item.imageLabel}</span>
                <h4>{item.title}</h4>
                <p>{item.description}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ── FAQ ────────────────────────────────────── */}
      <section className="sec" id="scope">
        <header className="sec__head">
          <p className="sec__kicker">FAQ</p>
          <h2 className="sec__h2">Common questions.</h2>
        </header>

        <div className="faq-grid">
          {FAQ_ITEMS.map((item) => (
            <div key={item.question} className="faq">
              <h3 className="faq__q">{item.question}</h3>
              <p className="faq__a">{item.answer}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ── Bottom CTA ─────────────────────────────── */}
      <section className="cta-banner">
        <h2>Ready to study smarter?</h2>
        <p>Download ObsidianOS and turn your vault into an exam system.</p>
        <a className="btn btn--white btn--lg" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
          <Download size={18} /> Get ObsidianOS
        </a>
      </section>

      <Footer onTos={goTos} />
    </div>
  );
}

/* ── Shared small components ──────────────────────── */

function NavBar({ onBrandClick }: { onBrandClick: () => void }) {
  return (
    <nav className="nav">
      <div className="nav__inner">
        <button className="nav__brand" onClick={onBrandClick} type="button">
          <BrandMark className="nav__logo" />
          <span>ObsidianOS</span>
        </button>
        <div className="nav__links">
          {LANDING_NAV.map((n) => <a key={n.id} href={`#${n.id}`}>{n.label}</a>)}
        </div>
        <a className="btn btn--white btn--sm" href={HERO_CTAS[0].href} target="_blank" rel="noreferrer">
          Download
        </a>
      </div>
    </nav>
  );
}

function Footer({ onTos }: { onTos: () => void }) {
  return (
    <footer className="footer">
      <div className="footer__inner">
        <div className="footer__left">
          <BrandMark className="footer__logo" />
          <span>© {new Date().getFullYear()} ObsidianOS</span>
        </div>
        <div className="footer__right">
          <button onClick={onTos} type="button">Terms of Service</button>
          {FOOTER_LINKS.map((l) => <a key={l.label} href={l.href} target="_blank" rel="noreferrer">{l.label}</a>)}
        </div>
      </div>
    </footer>
  );
}
