export type LandingLink = {
  label: string;
  href: string;
};

export type LandingCta = LandingLink & {
  note: string;
  variant: "primary" | "secondary" | "ghost";
};

export type LandingFeature = {
  eyebrow: string;
  title: string;
  description: string;
  bullets: string[];
};

export type LandingWorkflowStep = {
  id: string;
  label: string;
  title: string;
  description: string;
  outcome: string;
};

export type LandingShowcaseItem = {
  title: string;
  description: string;
  alt: string;
  imageSrc: string;
  imageLabel: string;
};

export type LandingFaqItem = {
  question: string;
  answer: string;
};

export type LandingSectionLink = {
  id: string;
  label: string;
};
