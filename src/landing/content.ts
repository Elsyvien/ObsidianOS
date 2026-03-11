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
];

export const HERO_CTAS: LandingCta[] = [
  {
    label: "Download",
    href: LANDING_LINKS.releases,
    note: "GitHub Releases",
    variant: "primary",
  },
  {
    label: "View Source",
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
  "Cross-platform desktop app",
  "Obsidian vault scanning",
  "Graph-driven exam prep",
  "AI-powered enrichment",
] as const;

export const FEATURE_CARDS: LandingFeature[] = [
  {
    eyebrow: "Vault scan",
    title: "Turn top-level course folders into study workspaces.",
    description:
      "ObsidianOS reads markdown notes from your vault, groups them by course, and builds a working library without changing your existing workflow.",
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
    question: "What platforms does ObsidianOS run on?",
    answer:
      "ObsidianOS is built with Tauri and runs on Windows, macOS, and Linux. The desktop app needs the Rust toolchain and WebView2 (Windows) or WebKitGTK (Linux) to build from source.",
  },
  {
    question: "Does it modify my Obsidian vault?",
    answer:
      "Only when you explicitly generate outputs. Scan and review are read-only. Flashcard files, revision notes, and exam artifacts are written to configurable output folders inside the vault.",
  },
  {
    question: "Does it need an internet connection?",
    answer:
      "Not for the core workflow. Vault scanning, graph analysis, and exam generation all run locally. An internet connection is only needed if you enable optional AI enrichment via OpenRouter or an OpenAI-compatible API.",
  },
  {
    question: "How does AI enrichment work?",
    answer:
      "You can optionally configure an OpenRouter or OpenAI-compatible endpoint in Settings. When enabled, it enriches note summaries, generates smarter flashcards, and provides formula briefings. All AI calls are opt-in and use your own API key.",
  },
  {
    question: "Can I use it alongside the Obsidian editor?",
    answer:
      "Yes. ObsidianOS reads the same vault folder Obsidian uses. You can edit notes in Obsidian and re-scan in ObsidianOS to pick up changes. There's no lock-in or sync conflict.",
  },
  {
    question: "Where do I download the app?",
    answer:
      "From the GitHub Releases page. Windows builds include an MSI or NSIS installer. For macOS and Linux, you can build from source using the instructions in the README.",
  },
];

export const FOOTER_LINKS: LandingLink[] = [
  { label: "GitHub Releases", href: LANDING_LINKS.releases },
  { label: "Repository", href: LANDING_LINKS.repository },
  { label: "README", href: LANDING_LINKS.readme },
];
