declare module "vitest/config" {
  export function defineConfig(config: any): any;
  const _default: any;
  export default _default;
}

declare module "react" {
  export const useEffect: any;
  export const useId: any;
  export type FC<P = any> = (props: P) => any;
  const React: any;
  export default React;
}

declare module "react/jsx-runtime" {
  export const jsx: any;
  export const jsxs: any;
  export const Fragment: any;
}

declare module "@testing-library/react" {
  export const render: any;
}

declare module "@testing-library/jest-dom" {}

declare module "@angular/core" {
  export function Component(options: any): ClassDecorator;
  export class ElementRef<T = any> {
    nativeElement: T;
    constructor(nativeElement: T);
  }
  export interface AfterViewInit {
    ngAfterViewInit(): void;
  }
  export function Input(options?: any): PropertyDecorator;
}

declare module "@angular/core/testing" {
  export const TestBed: any;
  export interface ComponentFixture<T> {
    componentInstance: T;
    nativeElement: any;
    detectChanges(): void;
  }
  export function waitForAsync(fn: () => Promise<void> | void): () => Promise<void>;
  export function getTestBed(): any;
}

declare module "@angular/platform-browser-dynamic/testing" {
  export const BrowserDynamicTestingModule: any;
  export function platformBrowserDynamicTesting(): any;
}

declare module "zone.js" {}
declare module "zone.js/testing" {}
