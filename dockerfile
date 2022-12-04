FROM fedora:37

RUN dnf install xorg-x11-server-Xvfb util-linux alsa-lib-devel libXcursor libXrandr libXi -y

ADD assets/ /opt/assets
ADD artificial_bike_racing /opt/artifical_bike_racing

RUN ls /opt


ENTRYPOINT ["sleep", "100000"]