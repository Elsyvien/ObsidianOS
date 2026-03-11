import examShowcase from "../assets/landing-exam-showcase.svg";
import formulaShowcase from "../assets/landing-formula-showcase.svg";
import overviewShowcase from "../assets/landing-overview-showcase.svg";
import type {
  LandingCta,
  LandingFaqItem,
  LandingFeature,
  LandingLink,
  LandingSectionLink,
  LandingShowcaseItem,
  LandingWorkflowStep,
} from "./types";

export const LANDING_LINKS = {
  releases: "https://github.com/Elsyvien/ObsidianOS/releases",
  repository: "https://github.com/Elsyvien/ObsidianOS",
  readme: "https://github.com/Elsyvien/ObsidianOS#readme",
} as const;

export const LANDING_NAV: LandingSectionLink[] = [
  { id: "features", label: "Features" },
  { id: "workflow", label: "Workflow" },
  { id: "showcase", label: "Showcase" },
  { id: "scope", label: "FAQ" },
  { id: "download", label: "Download" },
];

export const HERO_CTAS: LandingCta[] = [
  {
    label: "Download for Windows",
    href: LANDING_LINKS.releases,
    note: "GitHub Releases",
    variant: "primary",
  },
  {
    label: "View repository",
    href: LANDING_LINKS.repository,
    note: "Source, issues, changelog",
    variant: "secondary",
  },
  {
    label: "Read the README",
    href: LANDING_LINKS.readme,
    note: "Current scope and setup",
    variant: "ghost",
  },
];

export const HERO_PILLARS = [
  "Windows desktop app",
  "Obsidian vault scanning",
  "Graph-driven exam prep",
  "Static site only for product overview",
] as const;

export const FEATURE_CARDS: LandingFeature[] = [
  {
    eyebrow: "Vault scan",
    title: "Turn top-level course folders into study workspaces.",
    description:
      "ObsidianOS reads markdown notes from your vault, groups them by course, and builds a working library without changing your desktop runtime.",
    bullets: [
      "One connected Obsidian vault at a time",
      "Markdown-first indexing for .md files",
      "Course context from top-level folders",
    ],
  },
  {
    eyebrow: "Graph review",
    title: "See where your notes are connected and where they are weak.",
    description:
      "The dashboard surfaces weakly linked notes, concept coverage, graph density, and formula visibility so you can spend revision time where the structure actually breaks.",
    bullets: [
      "Weak-link detection for underconnected notes",
      "Coverage and graph health in one pass",
      "Recent notes and formula density at a glance",
    ],
  },
  {
    eyebrow: "Flashcards",
    title: "Generate review material back into the vault.",
    description:
      "Selected notes can be turned into markdown flashcards and optional CSV output for Anki-style import without moving your source material into a hosted service.",
    bullets: [
      "Markdown flashcard sets",
      "Optional CSV export",
      "Built from queued study notes",
    ],
  },
  {
    eyebrow: "Revision notes",
    title: "Write a focused revision note instead of another vague to-do list.",
    description:
      "Daily revision notes are generated from the scanned course context so the next session starts with concrete gaps, not a blank page.",
    bullets: [
      "Writes back into your vault",
      "Course-scoped review focus",
      "Keeps planning inside your note system",
    ],
  },
  {
    eyebrow: "Formula library",
    title: "Collect formulas with note context, not just a formula dump.",
    description:
      "Formula extraction rolls up where equations appear, which notes mention them, and what surrounding chunks need more practice before an exam.",
    bullets: [
      "Per-course formula library",
      "Linked notes and chunk previews",
      "Optional AI briefing on top",
    ],
  },
  {
    eyebrow: "Exam prep",
    title: "Queue source notes, generate tests, and feed mistakes back into revision.",
    description:
      "The exam flow is built for sprint, mock, and final runs so weak answers can be pushed back into the queue instead of disappearing after grading.",
    bullets: [
      "Question generation presets",
      "Review actions based on mistakes",
      "Designed for iterative prep loops",
    ],
  },
];

export const WORKFLOW_STEPS: LandingWorkflowStep[] = [
  {
    id: "01",
    label: "Connect",
    title: "Point the desktop app at an Obsidian vault.",
    description:
      "Start in the Tauri app, connect your vault path, and keep the source material where you already study.",
    outcome: "No browser app state, no hosted workspace clone.",
  },
  {
    id: "02",
    label: "Scan",
    title: "Index markdown into courses, concepts, graph links, and formulas.",
    description:
      "A scan creates the working overview for each course so the dashboard reflects structure, not just file counts.",
    outcome: "Coverage, weak notes, formulas, and countdowns update from the local workspace.",
  },
  {
    id: "03",
    label: "Review",
    title: "Find weak notes before they become exam surprises.",
    description:
      "Use graph gaps, note density, and AI/workspace views to decide which material needs linking, rewriting, or deeper recall practice.",
    outcome: "Revision effort follows actual weak structure instead of guesswork.",
  },
  {
    id: "04",
    label: "Generate",
    title: "Write flashcards, revision notes, and exam sets from selected material.",
    description:
      "Outputs are generated from the same course context, which keeps study artifacts grounded in the vault instead of scattered across tools.",
    outcome: "The vault becomes the record of both source notes and review outputs.",
  },
  {
    id: "05",
    label: "Prepare",
    title: "Use exam feedback to update the next revision cycle.",
    description:
      "Question results can push notes back into the queue, which closes the loop between mock exams and daily study.",
    outcome: "Exam prep becomes a repeatable system instead of a one-off cram session.",
  },
];

export const SHOWCASE_ITEMS: LandingShowcaseItem[] = [
  {
    title: "Overview board",
    description:
      "A static capture of the current course dashboard: coverage, weak-link cues, formulas, and countdown context in one place.",
    alt: "Overview showcase based on the current ObsidianOS browser preview dashboard.",
    imageSrc: overviewShowcase,
    imageLabel: "Current overview UI",
  },
  {
    title: "Formula workspace",
    description:
      "Formula rollups stay attached to notes and chunk previews, so practice starts from context instead of isolated equations.",
    alt: "Formula workspace showcase based on the current ObsidianOS formula UI.",
    imageSrc: formulaShowcase,
    imageLabel: "Formula library capture",
  },
  {
    title: "Exam engine",
    description:
      "Queue notes, generate a revision run, then route weak answers back into the study loop for the next session.",
    alt: "Exam workspace showcase based on the current ObsidianOS exam UI.",
    imageSrc: examShowcase,
    imageLabel: "Exam workflow capture",
  },
];

export const FAQ_ITEMS: LandingFaqItem[] = [
  {
    question: "Can I use ObsidianOS in the browser?",
    answer:
      "No. This site is a landing page only. The product itself runs as a Windows desktop app built with Tauri, and the browser build is just the public showcase.",
  },
  {
    question: "What does the app read today?",
    answer:
      "The current scope is one connected Obsidian vault at a time, with top-level folders treated as course spaces and markdown notes indexed from .md files.",
  },
  {
    question: "Does it rely on a hosted backend?",
    answer:
      "No backend is required for the core workflow. The current product uses local deterministic extraction, with optional OpenRouter or OpenAI-compatible refinement where configured.",
  },
  {
    question: "What outputs does it generate?",
    answer:
      "Today the app can produce markdown flashcards, optional Anki-style CSV export, and revision notes written back into the vault.",
  },
  {
    question: "What is the main download path?",
    answer:
      "The primary public download target is the GitHub Releases page for this repository.",
  },
];

export const FOOTER_LINKS: LandingLink[] = [
  { label: "GitHub Releases", href: LANDING_LINKS.releases },
  { label: "Repository", href: LANDING_LINKS.repository },
  { label: "README", href: LANDING_LINKS.readme },
];
