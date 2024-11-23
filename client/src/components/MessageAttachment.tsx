import { useEffect, useState } from 'react';

interface Props {
  fileName?: string;
  filePath?: string;
}

export default function MessageAttachment({ fileName, filePath }: Props) {
  const [fileType, setFileType] = useState<string | null>(null);
  const [fileSize, setFileSize] = useState<number | null>(null);

  useEffect(() => {
    if (filePath) {
      // Can't use HEAD request because of a bug in hyper, 
      // the lower level library used by the rust backend
      // reference: https://github.com/hyperium/hyper/issues/2427
      fetch(filePath, { method: 'GET' })
        .then(response => {
          const contentType = response.headers.get('Content-Type');
          const contentLength = response.headers.get('Content-Length');
          if (contentType) {
            setFileType(contentType);
          }
          if (contentLength) {
            setFileSize(parseInt(contentLength, 10));
          }
        })
        .catch(error => {
          console.error('Error fetching file metadata:', error);
        });
    }
  }, [filePath]);

  const renderAttachment = () => {
    if (!filePath || !fileType) return null;

    if (fileType.startsWith('image/')) {
      return <img className='max-w-full max-h-64 object-contain' src={filePath} alt={fileName} />;
    } else if (fileType.startsWith('video/')) {
      return <video className='max-w-full max-h-64 object-contain' controls>
        <source src={filePath} type={fileType} />
        Your browser does not support the video tag.
      </video>;
    } else if (fileType.startsWith('audio/')) {
      return <audio controls>
        <source src={filePath} type={fileType} />
        Your browser does not support the audio tag.
      </audio>;
    } else {
      return (
        <div className="flex space-x-4 items-center bg-opacity-40 bg-black rounded-lg px-6 py-4">
          <div className="flex-shrink-0 w-6 h-6">
            <a href={filePath} download>
              <img
                src="/download.svg"
                alt="Download"
                className="w-full h-full"
              />
            </a>
          </div>
          <div className="flex-grow">
            <span>{fileName}</span>
            <div className="text-sm text-gray-200 opacity-50">
              {fileType} - {fileSize ? (fileSize / 1024).toFixed(2) + ' KB' : 'Unknown size'}
            </div>
          </div>
        </div>
      );
    }
  };

  return (
    <div className="message-attachment">
      {renderAttachment()}
    </div>
  );
}
