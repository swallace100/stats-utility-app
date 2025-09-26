import { NavigationMenu, NavigationMenuList } from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";

export default function NavBar() {
  return (
    <header className="border-b">
      <div className="mx-auto max-w-6xl px-4 h-14 flex items-center justify-between">
        <a href="/" className="font-semibold">Stats Utility</a>
        <NavigationMenu>
          <NavigationMenuList className="gap-2">
            <a href="/jobs" className="text-sm hover:underline">Jobs</a>
            <a href="/upload" className="text-sm hover:underline">Upload</a>
          </NavigationMenuList>
        </NavigationMenu>
        <Button asChild size="sm"><a href="/docs">Docs</a></Button>
      </div>
    </header>
  );
}
