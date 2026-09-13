import Link from "next/link";

const links = [
  { href: "/", label: "Dashboard" },
  { href: "/twin", label: "Twin" },
  { href: "/incidents", label: "Incidents" },
];

export default function Nav() {
  return (
    <header className="border-b border-neutral-800 bg-neutral-950 text-white">
      <div className="max-w-6xl mx-auto px-8 py-4 flex items-center justify-between gap-6">
        <Link href="/" className="font-semibold tracking-tight">
          Industrial Intelligence
        </Link>
        <nav className="flex items-center gap-5 text-sm text-neutral-400">
          {links.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="hover:text-white"
            >
              {link.label}
            </Link>
          ))}
        </nav>
      </div>
    </header>
  );
}
