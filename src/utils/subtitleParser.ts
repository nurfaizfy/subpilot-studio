export interface SubtitleLine {
  id: string;
  start: number;
  end: number;
  text: string;
  style?: string;
}

export interface AssStyle {
  Name: string;
  Fontname: string;
  Fontsize: string;
  PrimaryColour: string;
  SecondaryColour: string;
  OutlineColour: string;
  BackColour: string;
  Bold: string;
  Italic: string;
  Underline: string;
  StrikeOut: string;
  ScaleX: string;
  ScaleY: string;
  Spacing: string;
  Angle: string;
  BorderStyle: string;
  Outline: string;
  Shadow: string;
  Alignment: string;
  MarginL: string;
  MarginR: string;
  MarginV: string;
  Encoding: string;
}

export interface AssData {
  scriptInfo: string[];
  styles: Record<string, AssStyle>;
  styleFormat: string[];
  eventsFormat: string[];
  lines: SubtitleLine[];
}

const generateId = () => Math.random().toString(36).substring(2, 15);

export const timeToSeconds = (timeStr: string): number => {
  const parts = timeStr.replace(',', '.').split(':');
  if (parts.length === 3) {
    const h = parseFloat(parts[0]);
    const m = parseFloat(parts[1]);
    const s = parseFloat(parts[2]);
    return h * 3600 + m * 60 + s;
  }
  return 0;
};

export const secondsToTime = (seconds: number, format: 'srt' | 'ass' = 'srt'): string => {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  const ms = Math.floor((seconds - Math.floor(seconds)) * 1000);
  const cs = Math.floor((seconds - Math.floor(seconds)) * 100);

  const pad = (num: number, size: number) => num.toString().padStart(size, '0');
  
  if (format === 'ass') {
    return `${h}:${pad(m, 2)}:${pad(s, 2)}.${pad(cs, 2)}`;
  }
  
  return `${pad(h, 2)}:${pad(m, 2)}:${pad(s, 2)},${pad(ms, 3)}`;
};

export const assColorToHex = (assColor: string) => {
  const clean = (assColor || '').replace(/&H/g, '').replace(/&/g, '');
  if (clean.length >= 6) {
    const idx = clean.length === 8 ? 2 : 0;
    const b = clean.substring(idx, idx + 2).padEnd(2, '0');
    const g = clean.substring(idx + 2, idx + 4).padEnd(2, '0');
    const r = clean.substring(idx + 4, idx + 6).padEnd(2, '0');
    let aStr = 'FF';
    if (clean.length === 8) {
      const a = clean.substring(0, 2);
      aStr = (255 - parseInt(a, 16)).toString(16).padStart(2, '0').toUpperCase();
    }
    return `#${r}${g}${b}${aStr}`;
  }
  return '#FFFFFFFF';
}

export const hexToAssColor = (hex: string) => {
  const r = (hex.substring(1, 3) || 'FF').padEnd(2, '0');
  const g = (hex.substring(3, 5) || 'FF').padEnd(2, '0');
  const b = (hex.substring(5, 7) || 'FF').padEnd(2, '0');
  let a = hex.substring(7, 9);
  if (!a || a.length < 2) a = 'FF';
  const assA = (255 - parseInt(a, 16)).toString(16).padStart(2, '0').toUpperCase();
  return `&H${assA}${b}${g}${r}`;
}

export class SubtitleParser {
  static parseSrt(content: string): SubtitleLine[] {
    const lines = content.replace(/\r\n/g, '\n').split('\n');
    const result: SubtitleLine[] = [];
    
    let currentLine: Partial<SubtitleLine> = {};
    let textLines: string[] = [];
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      
      if (line === '') {
        if (currentLine.start !== undefined && currentLine.end !== undefined) {
          currentLine.text = textLines.join('\n');
          currentLine.id = generateId();
          result.push(currentLine as SubtitleLine);
        }
        currentLine = {};
        textLines = [];
        continue;
      }
      
      if (line.includes('-->')) {
        const [start, end] = line.split('-->').map(s => s.trim());
        currentLine.start = timeToSeconds(start);
        currentLine.end = timeToSeconds(end);
      } else if (isNaN(Number(line))) {
        textLines.push(line);
      }
    }
    
    if (currentLine.start !== undefined && textLines.length > 0) {
      currentLine.text = textLines.join('\n');
      currentLine.id = generateId();
      result.push(currentLine as SubtitleLine);
    }
    
    return result;
  }

  static serializeSrt(lines: SubtitleLine[]): string {
    return lines.map((line, i) => {
      return `${i + 1}\n${secondsToTime(line.start, 'srt')} --> ${secondsToTime(line.end, 'srt')}\n${line.text}\n`;
    }).join('\n');
  }

  static parseAss(content: string): AssData {
    const lines = content.replace(/\r\n/g, '\n').split('\n');
    
    const assData: AssData = {
      scriptInfo: [],
      styles: {},
      styleFormat: [],
      eventsFormat: [],
      lines: []
    };

    let currentSection = '';

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;

      if (line.startsWith('[')) {
        currentSection = line;
        if (currentSection === '[Script Info]') assData.scriptInfo.push(line);
        continue;
      }

      if (currentSection === '[Script Info]') {
        assData.scriptInfo.push(line);
      } 
      else if (currentSection === '[V4+ Styles]' || currentSection === '[V4 Styles]') {
        if (line.startsWith('Format:')) {
          assData.styleFormat = line.substring(7).split(',').map(s => s.trim());
        } else if (line.startsWith('Style:')) {
          const values = line.substring(6).split(',').map(s => s.trim());
          const styleObj: any = {};
          assData.styleFormat.forEach((key, idx) => {
            styleObj[key] = values[idx] || '';
          });
          if (styleObj.Name) {
            assData.styles[styleObj.Name] = styleObj as AssStyle;
          }
        }
      } 
      else if (currentSection === '[Events]') {
        if (line.startsWith('Format:')) {
          assData.eventsFormat = line.substring(7).split(',').map(s => s.trim());
        } else if (line.startsWith('Dialogue:')) {
          const parts = line.substring(9).split(',');
          const numCols = assData.eventsFormat.length;
          
          const values = parts.slice(0, numCols - 1).map(s => s.trim());
          const text = parts.slice(numCols - 1).join(',').trim();
          
          const eventObj: any = {};
          assData.eventsFormat.forEach((key, idx) => {
            if (idx === numCols - 1) {
              eventObj[key] = text;
            } else {
              eventObj[key] = values[idx] || '';
            }
          });

          const start = timeToSeconds(eventObj.Start || '0:00:00.00');
          const end = timeToSeconds(eventObj.End || '0:00:00.00');
          const parsedText = (eventObj.Text || '').replace(/\\N/g, '\n');
          const style = eventObj.Style || 'Default';

          assData.lines.push({
            id: generateId(),
            start,
            end,
            text: parsedText,
            style
          });
        }
      }
    }
    
    if (!assData.styles['Default']) {
      assData.styleFormat = ['Name', 'Fontname', 'Fontsize', 'PrimaryColour', 'SecondaryColour', 'OutlineColour', 'BackColour', 'Bold', 'Italic', 'Underline', 'StrikeOut', 'ScaleX', 'ScaleY', 'Spacing', 'Angle', 'BorderStyle', 'Outline', 'Shadow', 'Alignment', 'MarginL', 'MarginR', 'MarginV', 'Encoding'];
      assData.styles['Default'] = {
        Name: 'Default', Fontname: 'Arial', Fontsize: '20', PrimaryColour: '&H00FFFFFF', SecondaryColour: '&H000000FF', OutlineColour: '&H00000000', BackColour: '&H00000000', Bold: '0', Italic: '0', Underline: '0', StrikeOut: '0', ScaleX: '100', ScaleY: '100', Spacing: '0', Angle: '0', BorderStyle: '1', Outline: '2', Shadow: '2', Alignment: '2', MarginL: '10', MarginR: '10', MarginV: '10', Encoding: '1'
      } as AssStyle;
    }
    if (assData.eventsFormat.length === 0) {
      assData.eventsFormat = ['Layer', 'Start', 'End', 'Style', 'Name', 'MarginL', 'MarginR', 'MarginV', 'Effect', 'Text'];
    }

    return assData;
  }

  static serializeAss(assData: AssData, globalStyles?: AssStyle[]): string {
    let output = '';

    if (assData.scriptInfo && assData.scriptInfo.length > 0) {
      output += assData.scriptInfo.join('\n') + '\n\n';
    } else {
      output += '[Script Info]\nScriptType: v4.00+\nPlayResX: 1280\nPlayResY: 720\n\n';
    }

    output += '[V4+ Styles]\n';
    output += 'Format: ' + assData.styleFormat.join(', ') + '\n';
    
    const mergedStyles = { ...assData.styles };
    if (globalStyles) {
      globalStyles.forEach(gs => {
        mergedStyles[gs.Name] = gs;
      });
    }

    for (const styleName in mergedStyles) {
      const style = mergedStyles[styleName];
      const values = assData.styleFormat.map(key => (style as any)[key] || '');
      output += 'Style: ' + values.join(',') + '\n';
    }
    output += '\n';

    output += '[Events]\n';
    output += 'Format: ' + assData.eventsFormat.join(', ') + '\n';
    assData.lines.forEach(line => {
      const eventObj: any = {
        Layer: '0',
        Start: secondsToTime(line.start, 'ass'),
        End: secondsToTime(line.end, 'ass'),
        Style: line.style || 'Default',
        Name: '',
        MarginL: '0',
        MarginR: '0',
        MarginV: '0',
        Effect: '',
        Text: line.text.replace(/\n/g, '\\N')
      };

      const values = assData.eventsFormat.map((key, idx) => {
        if (idx === assData.eventsFormat.length - 1) {
          return eventObj.Text;
        }
        return eventObj[key] !== undefined ? eventObj[key] : '';
      });

      output += 'Dialogue: ' + values.join(',') + '\n';
    });

    return output;
  }
}
