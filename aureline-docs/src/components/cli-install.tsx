import { Check, Copy } from 'lucide-react';
import { useRef, useState } from 'react';

const installers = [
  {
    label: 'macOS / Linux',
    command: 'curl -fsSL https://aureline.pixelscortex.com/install.sh | sh',
    href: 'https://aureline.pixelscortex.com/install.sh',
  },
  {
    label: 'Windows PowerShell',
    command: 'irm https://aureline.pixelscortex.com/install.ps1 | iex',
    href: 'https://aureline.pixelscortex.com/install.ps1',
  },
];

export function CliInstall() {
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);
  const resetTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  async function copyCommand(command: string) {
    await navigator.clipboard.writeText(command);
    setCopiedCommand(command);

    if (resetTimer.current) {
      clearTimeout(resetTimer.current);
    }

    resetTimer.current = setTimeout(() => setCopiedCommand(null), 2000);
  }

  return (
    <div className="not-prose my-6 grid gap-3">
      {installers.map((installer) => {
        const copied = copiedCommand === installer.command;

        return (
          <div
            className="rounded-xl border bg-fd-card p-4 text-fd-card-foreground shadow-sm"
            key={installer.label}
          >
            <div className="mb-3 flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-semibold">{installer.label}</p>
                <a
                  className="text-xs text-fd-muted-foreground underline underline-offset-4 hover:text-fd-foreground"
                  href={installer.href}
                  rel="noreferrer"
                  target="_blank"
                >
                  Open installer
                </a>
              </div>
            </div>
            <div className="flex flex-col gap-2 rounded-lg bg-fd-muted p-3 sm:flex-row sm:items-center sm:justify-between">
              <code className="overflow-x-auto text-sm text-fd-foreground">{installer.command}</code>
              <button
                aria-label={`Copy ${installer.label} install command`}
                className="inline-flex shrink-0 items-center justify-center gap-2 rounded-md border bg-fd-background px-3 py-2 text-sm font-medium text-fd-foreground transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground"
                onClick={() => void copyCommand(installer.command)}
                type="button"
              >
                {copied ? <Check className="size-4" /> : <Copy className="size-4" />}
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
