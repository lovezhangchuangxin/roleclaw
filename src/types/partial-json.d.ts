declare module "partial-json" {
  export const STR: number;
  export const NUM: number;
  export const ARR: number;
  export const OBJ: number;
  export const NULL: number;
  export const BOOL: number;
  export const NAN: number;
  export const INFINITY: number;
  export const _INFINITY: number;
  export const INF: number;
  export const SPECIAL: number;
  export const ATOM: number;
  export const COLLECTION: number;
  export const ALL: number;
  export function parse(jsonString: string, allowPartial?: number): unknown;
}
