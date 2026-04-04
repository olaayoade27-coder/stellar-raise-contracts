import React from 'react';
import { FrontendGlobalErrorBoundary } from '../components/frontend_global_error';

function MyApp({ Component, pageProps }: { Component: React.ComponentType<Record<string, unknown>>; pageProps: Record<string, unknown> }) {
  return (
    <FrontendGlobalErrorBoundary>
      <Component {...pageProps} />
    </FrontendGlobalErrorBoundary>
  );
}

export default MyApp;
