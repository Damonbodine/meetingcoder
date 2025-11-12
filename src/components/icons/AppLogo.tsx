import React from "react";

const AppLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <img
      src="/meetingcoderlogo.png"
      alt="MeetingCoder"
      width={width}
      height={height}
      className={className}
      style={{ objectFit: 'contain' }}
    />
  );
};

export default AppLogo;
