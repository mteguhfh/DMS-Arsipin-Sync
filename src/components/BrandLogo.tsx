import logo from '../assets/arsipin-logo.png'

interface Props {
  size?: number
}

/**
 * Brand lockup: the Arsipin mark inside a soft card, with a sync badge
 * pinned to the corner to signal this is the sync client.
 */
export default function BrandLogo({ size = 84 }: Props) {
  const badge = Math.round(size * 0.34)

  return (
    <div className="brand" style={{ '--brand-size': `${size}px`, '--badge-size': `${badge}px` } as React.CSSProperties}>
      <div className="brand-mark">
        <img src={logo} alt="Arsipin" width={size} height={size} />
        <span className="brand-badge" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4"
               strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 12a9 9 0 0 1-9 9 9 9 0 0 1-6.36-2.64" />
            <path d="M3 12a9 9 0 0 1 9-9 9 9 0 0 1 6.36 2.64" />
            <polyline points="21 3 21 9 15 9" />
            <polyline points="3 21 3 15 9 15" />
          </svg>
        </span>
      </div>
      <div className="brand-text">
        <span className="brand-name">Arsipin</span>
        <span className="brand-suffix">Sync</span>
      </div>
    </div>
  )
}
