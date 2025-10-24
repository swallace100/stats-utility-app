export default function NavBar() {
  return (
    <header className="sticky top-0 z-30 border-b border-neutral-200/70 bg-white/70 backdrop-blur-sm">
      <div className="mx-auto max-w-7xl h-12 px-4 flex items-center justify-between">
        {/* left placeholder for balance */}
        <div className="text-[12px] text-neutral-400 select-none"> </div>

        {/* right: single link */}
        <nav className="ml-auto">
          <a
            href="/jobs"
            className="text-[12px] font-medium text-neutral-600 hover:text-neutral-900 transition"
          >
            Job History
          </a>
        </nav>
      </div>
    </header>
  );
}
