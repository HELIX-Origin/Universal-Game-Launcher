export type Platform =
  | "steam"
  | "epic"
  | "gog"
  | "humble"
  | "itch"
  | "ubisoft"
  | "ea"
  | "origin"
  | "xbox"
  | "amazon"
  | "battle-net"
  | "lutris"
  | "geforce-now"
  | "xcloud"
  | "local";

export interface LaunchTarget {
  kind: "uri" | "executable";
}

export interface GameEntry {
  id: string;
  title: string;
  platform: Platform;
  installed: boolean;
  install: LaunchTarget | null;
  coverUrl: string | null;
  heroUrl: string | null;
  installDir: string | null;
  favorite: boolean;
  hidden: boolean;
  custom: boolean;
}

export interface PlatformStatus {
  platform: Platform;
  gameCount: number;
  error: string | null;
}

export interface LibrarySnapshot {
  games: GameEntry[];
  platforms: PlatformStatus[];
  scannedAt: number;
}

export type Filter = "all" | "installed" | "favorites" | "hidden";
export type GameActionCommand = "launch_game" | "install_game";

export function gameActionCommand(game: GameEntry): GameActionCommand | null {
  if (game.installed) return "launch_game";
  if (game.install) return "install_game";
  return null;
}

export function filterLibraryGames(
  games: GameEntry[],
  query: string,
  filter: Filter,
  platformNames: Record<Platform, string>,
): GameEntry[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();

  return games
    .filter((game) => {
      if (filter === "hidden" ? !game.hidden : game.hidden) return false;
      if (filter === "installed" && !game.installed) return false;
      if (filter === "favorites" && !game.favorite) return false;
      if (!normalizedQuery) return true;

      return (
        game.title.toLocaleLowerCase().includes(normalizedQuery) ||
        platformNames[game.platform].toLocaleLowerCase().includes(normalizedQuery)
      );
    })
    .sort((a, b) => a.title.localeCompare(b.title));
}
