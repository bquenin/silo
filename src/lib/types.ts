export type Faction =
  | 'GDI' | 'Nod' | 'Sc'
  | 'BH' | 'MoK' | 'ST' | 'ZCM' | 'R17' | 'T59'
  | 'Rnd' | 'Obs';

export interface Player {
  slot: number;
  name: string;
  chosen: Faction | string;
  actual: Faction | string;
}

export interface Replay {
  id: string;
  file: string;
  map: string;
  n_players: number;
  duration_s?: number;
  recorded_at?: string;
  players: Player[];
  // derived fields
  bro_alias?: string;
  bro_actual?: Faction | string;
  opponent_name?: string;
  opponent_actual?: Faction | string;
}

export const FACTION_LABEL: Record<string, string> = {
  GDI: 'GDI', Nod: 'Nod', Sc: 'Scrin',
  BH: 'Black Hand', MoK: 'Marked of Kane',
  ST: 'Steel Talons', ZCM: 'ZOCOM',
  R17: 'Reaper-17', T59: 'Traveler-59',
  Rnd: 'Random', Obs: 'Observer',
};

export const FACTION_COLOR: Record<string, string> = {
  GDI: '#e6c34a',
  Nod: '#d44848',
  Sc: '#4ad4cf',
  BH: '#a83232',
  MoK: '#b04848',
  ST: '#c4a93e',
  ZCM: '#e0d36b',
  R17: '#3a9c97',
  T59: '#6ad4c4',
  Rnd: '#8b8b91',
  Obs: '#56565c',
};
