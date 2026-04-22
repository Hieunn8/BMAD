import type { BrandPreset } from "../start-flow/types";

type PresetCardProps = {
  isSelected: boolean;
  preset: BrandPreset;
};

export function PresetCard({ isSelected, preset }: PresetCardProps) {
  return (
    <section className={`preset-card${isSelected ? " preset-card--selected" : ""}`}>
      <div className="preset-card__hero">
        <div>
          <p className="preset-card__eyebrow">Brand Preset</p>
          <h3>{preset.brandName}</h3>
        </div>
        <div className="preset-card__logo">
          <span>Logo mac dinh</span>
          <code>{preset.defaultLogoPath}</code>
        </div>
      </div>

      <dl className="preset-card__details">
        <div>
          <dt>Audio policy</dt>
          <dd>{preset.audioReplacementPolicy}</dd>
        </div>
        <div>
          <dt>Subtitle style</dt>
          <dd>{preset.subtitleStylePreset}</dd>
        </div>
        <div>
          <dt>Layout rules</dt>
          <dd>{preset.layoutRules}</dd>
        </div>
        <div>
          <dt>Export preset</dt>
          <dd>{preset.exportPreset}</dd>
        </div>
      </dl>

      <p className="preset-card__notes">{preset.notes}</p>
    </section>
  );
}
