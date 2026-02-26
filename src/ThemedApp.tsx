import React from 'react';
import { FluentProvider, webDarkTheme, webLightTheme } from '@fluentui/react-components';
import App from './App';

const colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');

function ThemedApp() {
  const [isDark, setIsDark] = React.useState(colorSchemeQuery.matches);

  React.useEffect(() => {
    const handleChange = (event: MediaQueryListEvent) => {
      setIsDark(event.matches);
    };

    colorSchemeQuery.addEventListener('change', handleChange);
    return () => {
      colorSchemeQuery.removeEventListener('change', handleChange);
    };
  }, []);

  return (
    <FluentProvider theme={isDark ? webDarkTheme : webLightTheme}>
      <App />
    </FluentProvider>
  );
}

export default ThemedApp;
